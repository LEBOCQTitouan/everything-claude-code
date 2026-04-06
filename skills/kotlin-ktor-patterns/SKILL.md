---
name: kotlin-ktor-patterns
description: Ktor server patterns including routing DSL, plugins, authentication, Koin DI, kotlinx.serialization, WebSockets, and testApplication testing.
origin: ECC
---

# Ktor Server Patterns

## When to Activate

- Building Ktor HTTP servers or REST APIs
- Configuring plugins (Auth, CORS, ContentNegotiation, StatusPages)
- Setting up Koin DI or WebSockets
- Writing testApplication integration tests

## Project Layout

```text
src/main/kotlin/com/example/
├── Application.kt
├── plugins/ (Routing, Serialization, Authentication, StatusPages, CORS)
├── routes/ (UserRoutes, AuthRoutes, HealthRoutes)
├── models/ (domain models, response envelopes)
├── services/ (business logic)
├── repositories/ (data access)
└── di/ (Koin modules)
```

```kotlin
fun main() {
    embeddedServer(Netty, port = 8080, module = Application::module).start(wait = true)
}

fun Application.module() {
    configureSerialization()
    configureAuthentication()
    configureStatusPages()
    configureCORS()
    configureDI()
    configureRouting()
}
```

## Routing DSL

```kotlin
fun Route.userRoutes() {
    val userService by inject<UserService>()
    route("/users") {
        get { call.respond(userService.getAll()) }
        get("/{id}") {
            val id = call.parameters["id"] ?: return@get call.respond(HttpStatusCode.BadRequest, "Missing id")
            val user = userService.getById(id) ?: return@get call.respond(HttpStatusCode.NotFound)
            call.respond(user)
        }
        post {
            val request = call.receive<CreateUserRequest>()
            call.respond(HttpStatusCode.Created, userService.create(request))
        }
        put("/{id}") {
            val id = call.parameters["id"] ?: return@put call.respond(HttpStatusCode.BadRequest, "Missing id")
            val request = call.receive<UpdateUserRequest>()
            val user = userService.update(id, request) ?: return@put call.respond(HttpStatusCode.NotFound)
            call.respond(user)
        }
        delete("/{id}") {
            val id = call.parameters["id"] ?: return@delete call.respond(HttpStatusCode.BadRequest, "Missing id")
            if (userService.delete(id)) call.respond(HttpStatusCode.NoContent)
            else call.respond(HttpStatusCode.NotFound)
        }
    }
}

// Protected routes
authenticate("jwt") {
    post { /* requires auth */ }
}
```

## Serialization

```kotlin
fun Application.configureSerialization() {
    install(ContentNegotiation) {
        json(Json {
            prettyPrint = true; isLenient = false; ignoreUnknownKeys = true
            encodeDefaults = true; explicitNulls = false
        })
    }
}

@Serializable
data class ApiResponse<T>(val success: Boolean, val data: T? = null, val error: String? = null) {
    companion object {
        fun <T> ok(data: T) = ApiResponse(success = true, data = data)
        fun <T> error(message: String) = ApiResponse<T>(success = false, error = message)
    }
}
```

## JWT Authentication

```kotlin
fun Application.configureAuthentication() {
    val jwtSecret = environment.config.property("jwt.secret").getString()
    val jwtIssuer = environment.config.property("jwt.issuer").getString()
    val jwtAudience = environment.config.property("jwt.audience").getString()

    install(Authentication) {
        jwt("jwt") {
            verifier(JWT.require(Algorithm.HMAC256(jwtSecret)).withAudience(jwtAudience).withIssuer(jwtIssuer).build())
            validate { credential ->
                if (credential.payload.audience.contains(jwtAudience)) JWTPrincipal(credential.payload) else null
            }
            challenge { _, _ -> call.respond(HttpStatusCode.Unauthorized, ApiResponse.error<Unit>("Invalid or expired token")) }
        }
    }
}

fun ApplicationCall.userId(): String =
    principal<JWTPrincipal>()?.payload?.getClaim("userId")?.asString()
        ?: throw AuthenticationException("No userId in token")
```

## Status Pages (Error Handling)

```kotlin
fun Application.configureStatusPages() {
    install(StatusPages) {
        exception<ContentTransformationException> { call, cause ->
            call.respond(HttpStatusCode.BadRequest, ApiResponse.error<Unit>("Invalid request body: ${cause.message}"))
        }
        exception<IllegalArgumentException> { call, cause ->
            call.respond(HttpStatusCode.BadRequest, ApiResponse.error<Unit>(cause.message ?: "Bad request"))
        }
        exception<NotFoundException> { call, cause ->
            call.respond(HttpStatusCode.NotFound, ApiResponse.error<Unit>(cause.message ?: "Not found"))
        }
        exception<Throwable> { call, cause ->
            call.application.log.error("Unhandled exception", cause)
            call.respond(HttpStatusCode.InternalServerError, ApiResponse.error<Unit>("Internal server error"))
        }
    }
}
```

## Koin DI

```kotlin
val appModule = module {
    single<Database> { DatabaseFactory.create(get()) }
    single<UserRepository> { ExposedUserRepository(get()) }
    single { UserService(get()) }
    single { AuthService(get(), get()) }
}

fun Application.configureDI() { install(Koin) { modules(appModule) } }
```

## WebSockets

```kotlin
fun Application.configureWebSockets() {
    install(WebSockets) {
        pingPeriod = 15.seconds; timeout = 15.seconds
        maxFrameSize = 64 * 1024; masking = false
    }
}

fun Route.chatRoutes() {
    val connections = Collections.synchronizedSet<Connection>(LinkedHashSet())
    webSocket("/chat") {
        val thisConnection = Connection(this)
        connections += thisConnection
        try {
            send("Connected! Users online: ${connections.size}")
            for (frame in incoming) {
                frame as? Frame.Text ?: continue
                val message = ChatMessage(thisConnection.name, frame.readText())
                synchronized(connections) { connections.toList() }.forEach { it.session.send(Json.encodeToString(message)) }
            }
        } finally { connections -= thisConnection }
    }
}
```

## testApplication Testing

```kotlin
class UserRoutesTest : FunSpec({
    test("GET /users returns list") {
        testApplication {
            application {
                install(Koin) { modules(testModule) }
                configureSerialization(); configureRouting()
            }
            val response = client.get("/users")
            response.status shouldBe HttpStatusCode.OK
        }
    }

    test("POST /users creates user") {
        testApplication {
            application {
                install(Koin) { modules(testModule) }
                configureSerialization(); configureStatusPages(); configureRouting()
            }
            val client = createClient {
                install(io.ktor.client.plugins.contentnegotiation.ContentNegotiation) { json() }
            }
            val response = client.post("/users") {
                contentType(ContentType.Application.Json)
                setBody(CreateUserRequest("Alice", "alice@example.com"))
            }
            response.status shouldBe HttpStatusCode.Created
        }
    }

    test("protected route requires JWT") {
        testApplication {
            application { install(Koin) { modules(testModule) }; configureSerialization(); configureAuthentication(); configureRouting() }
            client.post("/users") { contentType(ContentType.Application.Json); setBody(CreateUserRequest("Alice", "a@b.c")) }.status shouldBe HttpStatusCode.Unauthorized
        }
    }
})
```

## Quick Reference

| Pattern | Description |
|---------|-------------|
| `route("/path") { get { } }` | Route grouping |
| `call.receive<T>()` | Deserialize request body |
| `call.respond(status, body)` | Send response |
| `call.parameters["id"]` | Path parameters |
| `install(Plugin) { }` | Configure plugin |
| `authenticate("name") { }` | Protect routes |
| `by inject<T>()` | Koin DI |
| `testApplication { }` | Integration testing |

Keep routes thin, push logic to services, use Koin for DI. Test with `testApplication` for full integration coverage.
