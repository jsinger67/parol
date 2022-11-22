/// Pops an item from the item stack.
/// Is not intended to be called by anyone else but the auto generated adapter grammar!
#[macro_export]
macro_rules! pop_item {
    ($self:ident, $name:ident, $type:ident, $context:ident) => {
        if let Some(ASTType::$type($name)) = $self.pop($context) {
            $name
        } else {
            bail!("{}: Expecting ASTType::{}", stringify!($type), $context);
        }
    };
}

/// Pops a vector item from the item stack and reverses it
/// Is not intended to be called by anyone else but the auto generated adapter grammar!
#[macro_export]
macro_rules! pop_and_reverse_item {
    ($self:ident, $name:ident, $type:ident, $context:ident) => {
        if let Some(ASTType::$type(mut $name)) = $self.pop($context) {
            $name.reverse();
            $name
        } else {
            bail!("{}: Expecting ASTType::{}", stringify!($type), $context);
        }
    };
}
