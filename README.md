# Redo
[![Build Status](https://travis-ci.org/evenorog/redo.svg?branch=master)](https://travis-ci.org/evenorog/redo)
[![Crates.io](https://img.shields.io/crates/v/redo.svg)](https://crates.io/crates/redo)
[![Docs](https://docs.rs/redo/badge.svg)](https://docs.rs/redo)

An undo/redo library with static dispatch and manual command merging.
It uses the [Command Pattern] where the user implements the `Command` trait for a command.

## Redo vs Undo
|                 | Redo             | Undo            |
|-----------------|------------------|-----------------|
| Dispatch        | [Static]         | [Dynamic]       |
| Command Merging | [Manual][manual] | [Auto][auto]    |

Both supports command merging but [`undo`] will automatically merge commands with the same id
while in `redo` you need to implement the merge method yourself.

## Examples
```rust
use redo::{Command, Stack};

#[derive(Debug)]
struct Push(char);

impl Command<String> for Push {
    type Err = &'static str;
    
    fn redo(&mut self, s: &mut String) -> Result<(), &'static str> {
        s.push(self.0);
        Ok(())
    }
    
    fn undo(&mut self, s: &mut String) -> Result<(), &'static str> {
        self.0 = s.pop().ok_or("`String` is unexpectedly empty")?;
        Ok(())
    }
}

fn foo() -> Result<(), (Push, &'static str)> {
    let mut stack = Stack::new(String::new());
    
    stack.push(Push('a'))?;
    stack.push(Push('b'))?;
    stack.push(Push('c'))?;
    
    assert_eq!(stack.as_receiver(), "abc");
    
    let c = stack.pop().unwrap()?;
    let b = stack.pop().unwrap()?;
    let a = stack.pop().unwrap()?;
    
    assert_eq!(stack.into_receiver(), "");
    Ok(())
}
```

[Command Pattern]: https://en.wikipedia.org/wiki/Command_pattern
[auto]: https://docs.rs/undo/0.8.1/undo/trait.UndoCmd.html#method.id
[manual]: trait.RedoCmd.html#method.merge
[Static]: https://doc.rust-lang.org/stable/book/trait-objects.html#static-dispatch
[Dynamic]: https://doc.rust-lang.org/stable/book/trait-objects.html#dynamic-dispatch
[`undo`]: https://crates.io/crates/undo
