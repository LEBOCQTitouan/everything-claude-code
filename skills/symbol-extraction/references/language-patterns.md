# Language-Specific Export/Import Patterns

Reference material for the symbol-extraction skill. Contains grep-ready patterns for identifying public symbols across languages.

## TypeScript / JavaScript

### Export Patterns

```
# Named exports
export function NAME
export const NAME
export class NAME
export type NAME
export interface NAME
export enum NAME
export abstract class NAME

# Default exports
export default function NAME
export default class NAME
export default NAME

# CommonJS
module.exports = { ... }
module.exports.NAME = ...
exports.NAME = ...

# Re-exports
export { NAME } from './module'
export { NAME as ALIAS } from './module'
export * from './module'
export * as NAMESPACE from './module'
```

### Import Patterns

```
import { NAME } from './module'
import NAME from './module'
import * as NS from './module'
const NAME = require('./module')
const { NAME } = require('./module')
```

### Doc Comment Format

```typescript
/**
 * Description of the symbol.
 * @param paramName - Description
 * @returns Description of return value
 * @throws {ErrorType} When condition
 * @example
 * const result = myFunction('input');
 * @deprecated Use newFunction instead
 */
```

## Python

### Export Patterns

```
# Explicit exports
__all__ = ['Name1', 'Name2']

# Implicit: top-level definitions without _ prefix
def public_function():
class PublicClass:
PUBLIC_CONSTANT = ...

# Internal: _ prefix
def _private_function():
class _PrivateClass:
```

### Import Patterns

```
import module
from module import Name
from module import Name as Alias
from package.module import Name
```

### Doc Comment Format

```python
def function(param: str) -> int:
    """Description of the function.

    Args:
        param: Description of param.

    Returns:
        Description of return value.

    Raises:
        ValueError: When condition.

    Example:
        >>> function('input')
        42
    """
```

## Go

### Export Patterns

```
# Capitalized = exported
func PublicFunction()
type PublicType struct
type PublicInterface interface
var PublicVar
const PublicConst

# Lowercase = unexported
func privateFunction()
type privateType struct
```

### Import Patterns

```
import "package"
import alias "package"
import (
    "package1"
    "package2"
)
```

### Doc Comment Format

```go
// PublicFunction does something important.
// It takes a parameter and returns a result.
func PublicFunction(param string) (int, error) {
```

## Rust

### Export Patterns

```
pub fn public_function()
pub struct PublicStruct
pub enum PublicEnum
pub trait PublicTrait
pub type PublicAlias = ...
pub const PUBLIC_CONST: Type = ...
pub mod public_module

# Crate-level visibility
pub(crate) fn crate_function()
pub(super) fn parent_module_function()
```

### Import Patterns

```
use crate::module::Name;
use super::Name;
use external_crate::Name;
use crate::module::{Name1, Name2};
use crate::module::*;
```

### Doc Comment Format

```rust
/// Description of the symbol.
///
/// # Arguments
///
/// * `param` - Description of param
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Returns `ErrorType` when condition
///
/// # Examples
///
/// ```
/// let result = public_function("input");
/// ```
pub fn public_function(param: &str) -> Result<i32, Error> {
```

## Java

### Export Patterns

```
public class ClassName
public interface InterfaceName
public enum EnumName
public void methodName()
public static final Type CONSTANT_NAME
public record RecordName(...)
```

### Doc Comment Format

```java
/**
 * Description of the symbol.
 *
 * @param paramName description
 * @return description of return value
 * @throws ExceptionType when condition
 */
```
