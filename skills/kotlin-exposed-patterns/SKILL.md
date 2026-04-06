---
name: kotlin-exposed-patterns
description: JetBrains Exposed ORM patterns including DSL queries, DAO pattern, transactions, HikariCP connection pooling, Flyway migrations, and repository pattern.
origin: ECC
---

# Kotlin Exposed Patterns

## When to Use

- Database access with Exposed (DSL or DAO)
- HikariCP connection pooling, Flyway migrations
- Repository pattern with Exposed, JSON columns

## Database Setup

### HikariCP + Flyway

```kotlin
object DatabaseFactory {
    fun create(config: DatabaseConfig): Database {
        val hikariConfig = HikariConfig().apply {
            driverClassName = config.driver
            jdbcUrl = config.url
            username = config.username
            password = config.password
            maximumPoolSize = config.maxPoolSize
            isAutoCommit = false
            transactionIsolation = "TRANSACTION_READ_COMMITTED"
            validate()
        }
        return Database.connect(HikariDataSource(hikariConfig))
    }
}

fun runMigrations(config: DatabaseConfig) {
    Flyway.configure()
        .dataSource(config.url, config.username, config.password)
        .locations("classpath:db/migration")
        .baselineOnMigrate(true)
        .load()
        .migrate()
}
```

## Table Definitions

```kotlin
object UsersTable : UUIDTable("users") {
    val name = varchar("name", 100)
    val email = varchar("email", 255).uniqueIndex()
    val role = enumerationByName<Role>("role", 20)
    val metadata = jsonb<UserMetadata>("metadata", Json.Default).nullable()
    val createdAt = timestampWithTimeZone("created_at").defaultExpression(CurrentTimestampWithTimeZone)
    val updatedAt = timestampWithTimeZone("updated_at").defaultExpression(CurrentTimestampWithTimeZone)
}

object OrdersTable : UUIDTable("orders") {
    val userId = uuid("user_id").references(UsersTable.id)
    val status = enumerationByName<OrderStatus>("status", 20)
    val totalAmount = long("total_amount")
    val currency = varchar("currency", 3)
    val createdAt = timestampWithTimeZone("created_at").defaultExpression(CurrentTimestampWithTimeZone)
}

// Composite key
object UserRolesTable : Table("user_roles") {
    val userId = uuid("user_id").references(UsersTable.id, onDelete = ReferenceOption.CASCADE)
    val roleId = uuid("role_id").references(RolesTable.id, onDelete = ReferenceOption.CASCADE)
    override val primaryKey = PrimaryKey(userId, roleId)
}
```

## DSL Queries

### CRUD

```kotlin
suspend fun insertUser(name: String, email: String, role: Role): UUID =
    newSuspendedTransaction {
        UsersTable.insertAndGetId {
            it[UsersTable.name] = name
            it[UsersTable.email] = email
            it[UsersTable.role] = role
        }.value
    }

suspend fun findUserById(id: UUID): UserRow? =
    newSuspendedTransaction {
        UsersTable.selectAll().where { UsersTable.id eq id }.map { it.toUser() }.singleOrNull()
    }

suspend fun updateUserEmail(id: UUID, newEmail: String): Boolean =
    newSuspendedTransaction {
        UsersTable.update({ UsersTable.id eq id }) {
            it[email] = newEmail
            it[updatedAt] = CurrentTimestampWithTimeZone
        } > 0
    }

suspend fun deleteUser(id: UUID): Boolean =
    newSuspendedTransaction { UsersTable.deleteWhere { UsersTable.id eq id } > 0 }

private fun ResultRow.toUser() = UserRow(
    id = this[UsersTable.id].value, name = this[UsersTable.name],
    email = this[UsersTable.email], role = this[UsersTable.role],
    metadata = this[UsersTable.metadata],
    createdAt = this[UsersTable.createdAt], updatedAt = this[UsersTable.updatedAt],
)
```

### Joins, Aggregation, Search

```kotlin
// Join
suspend fun findOrdersWithUser(userId: UUID): List<OrderWithUser> =
    newSuspendedTransaction {
        (OrdersTable innerJoin UsersTable).selectAll()
            .where { OrdersTable.userId eq userId }
            .orderBy(OrdersTable.createdAt, SortOrder.DESC)
            .map { row -> OrderWithUser(row[OrdersTable.id].value, row[OrdersTable.status], row[OrdersTable.totalAmount], row[UsersTable.name]) }
    }

// Aggregation
suspend fun countUsersByRole(): Map<Role, Long> =
    newSuspendedTransaction {
        UsersTable.select(UsersTable.role, UsersTable.id.count())
            .groupBy(UsersTable.role)
            .associate { it[UsersTable.role] to it[UsersTable.id.count()] }
    }

// LIKE — always escape user input to prevent wildcard injection
private fun escapeLikePattern(input: String): String =
    input.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_")

suspend fun searchUsers(query: String): List<UserRow> =
    newSuspendedTransaction {
        val sanitized = escapeLikePattern(query.lowercase())
        UsersTable.selectAll().where {
            (UsersTable.name.lowerCase() like "%${sanitized}%") or
                (UsersTable.email.lowerCase() like "%${sanitized}%")
        }.map { it.toUser() }
    }
```

### Pagination & Batch Operations

```kotlin
data class Page<T>(val data: List<T>, val total: Long, val page: Int, val limit: Int) {
    val totalPages: Int get() = ((total + limit - 1) / limit).toInt()
    val hasNext: Boolean get() = page < totalPages
}

suspend fun findUsersPaginated(page: Int, limit: Int): Page<UserRow> =
    newSuspendedTransaction {
        val total = UsersTable.selectAll().count()
        val data = UsersTable.selectAll()
            .orderBy(UsersTable.createdAt, SortOrder.DESC)
            .limit(limit).offset(((page - 1) * limit).toLong())
            .map { it.toUser() }
        Page(data, total, page, limit)
    }

suspend fun insertUsers(users: List<CreateUserRequest>): List<UUID> =
    newSuspendedTransaction {
        UsersTable.batchInsert(users) { user ->
            this[UsersTable.name] = user.name
            this[UsersTable.email] = user.email
            this[UsersTable.role] = user.role
        }.map { it[UsersTable.id].value }
    }
```

## DAO Pattern

```kotlin
class UserEntity(id: EntityID<UUID>) : UUIDEntity(id) {
    companion object : UUIDEntityClass<UserEntity>(UsersTable)
    var name by UsersTable.name
    var email by UsersTable.email
    var role by UsersTable.role
    var metadata by UsersTable.metadata
    val orders by OrderEntity referrersOn OrdersTable.userId

    fun toModel(): User = User(id.value, name, email, role, metadata, createdAt, updatedAt)
}

suspend fun createUser(request: CreateUserRequest): User =
    newSuspendedTransaction {
        UserEntity.new { name = request.name; email = request.email; role = request.role }.toModel()
    }
```

## Repository Pattern

```kotlin
interface UserRepository {
    suspend fun findById(id: UUID): User?
    suspend fun findByEmail(email: String): User?
    suspend fun findAll(page: Int, limit: Int): Page<User>
    suspend fun create(request: CreateUserRequest): User
    suspend fun update(id: UUID, request: UpdateUserRequest): User?
    suspend fun delete(id: UUID): Boolean
    suspend fun count(): Long
}

class ExposedUserRepository(private val database: Database) : UserRepository {
    override suspend fun findById(id: UUID): User? =
        newSuspendedTransaction(db = database) {
            UsersTable.selectAll().where { UsersTable.id eq id }.map { it.toUser() }.singleOrNull()
        }
    // Other methods follow the same pattern: newSuspendedTransaction(db = database) { ... }
}
```

## JSON Columns (JSONB)

```kotlin
inline fun <reified T : Any> Table.jsonb(name: String, json: Json): Column<T> =
    registerColumn(name, object : ColumnType<T>() {
        override fun sqlType() = "JSONB"
        override fun valueFromDB(value: Any): T = when (value) {
            is String -> json.decodeFromString(value)
            is PGobject -> json.decodeFromString(value.value ?: throw IllegalArgumentException("null PGobject"))
            else -> throw IllegalArgumentException("Unexpected value: $value")
        }
        override fun notNullValueToDB(value: T): Any =
            PGobject().apply { type = "jsonb"; this.value = json.encodeToString(value) }
    })
```

## Testing

```kotlin
class UserRepositoryTest : FunSpec({
    lateinit var database: Database
    lateinit var repository: UserRepository

    beforeSpec {
        database = Database.connect("jdbc:h2:mem:test;DB_CLOSE_DELAY=-1;MODE=PostgreSQL", "org.h2.Driver")
        transaction(database) { SchemaUtils.create(UsersTable) }
        repository = ExposedUserRepository(database)
    }

    beforeTest { transaction(database) { UsersTable.deleteAll() } }

    test("create and find user") {
        val user = repository.create(CreateUserRequest("Alice", "alice@example.com"))
        user.name shouldBe "Alice"
        repository.findById(user.id) shouldBe user
    }
})
```

## Gradle Dependencies

```kotlin
dependencies {
    implementation("org.jetbrains.exposed:exposed-core:1.0.0")
    implementation("org.jetbrains.exposed:exposed-dao:1.0.0")
    implementation("org.jetbrains.exposed:exposed-jdbc:1.0.0")
    implementation("org.jetbrains.exposed:exposed-kotlin-datetime:1.0.0")
    implementation("org.jetbrains.exposed:exposed-json:1.0.0")
    implementation("org.postgresql:postgresql:42.7.5")
    implementation("com.zaxxer:HikariCP:6.2.1")
    implementation("org.flywaydb:flyway-core:10.22.0")
    implementation("org.flywaydb:flyway-database-postgresql:10.22.0")
    testImplementation("com.h2database:h2:2.3.232")
}
```

## Quick Reference

| Pattern | Description |
|---------|-------------|
| `newSuspendedTransaction { }` | Coroutine-safe transaction |
| `Table.selectAll().where { }` | Query with conditions |
| `Table.insertAndGetId { }` | Insert and return ID |
| `Table.batchInsert(items) { }` | Bulk insert |
| `innerJoin` / `leftJoin` | Join tables |
| DSL style | Direct SQL-like queries |
| DAO style | Entity lifecycle management |

Use DSL for simple queries, DAO for entity lifecycle. Always use `newSuspendedTransaction` for coroutine support. Wrap queries behind a repository interface for testability.
