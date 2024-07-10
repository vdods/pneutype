# pneutype

Low-boilerplate creation of newtypes defined by validation constraints.

## Synopsis

Easily create newtypes analogous to `String` and `&str` that obey a particular validation constraint.  See `pneutype` crate docs.

## License

[MIT](LICENSE)

## To-dos

-   Add pneutypes analogous `Vec<T>` and `&[T]`.
-   Add feature for serde, and formally implement deserialization.
-   Get `Cow` deserializing with borrow for pneutypes.
-   Maybe make it possible to have a free-standing `PneuString`.  Though this might better just be done by hand.
-   Do an analysis of if this is a zero-overhead abstraction.  In particular, want to show that optimized code inlines everything to be equivalent to use of `String` and `&str` (apart from calls to `validate`).
