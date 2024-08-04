# pneutype

Low-boilerplate creation of newtypes defined by validation constraints.

## Synopsis

Easily create newtypes analogous to `String` and `str` that obey a particular validation constraint.  See `pneutype` crate docs.

## License

[MIT](LICENSE)

## To-dos

-   Update documentation to include support for generics and the `serialize` attribute.
-   Finish support for generics in `PneuString` and `PneuStr` -- in particular, get rid of `'static` bound requirement.
-   Don't bother with `str_field` and `string_field`, just require that a `PneuStr` has the form `struct XyzStr(str);` or `struct XyzStr<T>(std::marker::PhantomData<T>, str);` and analogous for `PneuString`.
-   Add pneutypes analogous `Vec<T>` and `[T]`.
-   Add pneutypes over `T` (and whatever the str/slice equivalent would be -- a reference?).
-   Get `Cow` deserializing with borrow for pneutypes.
-   Maybe make it possible to have a free-standing `PneuString` -- this would mean requiring impl of `Validate` and not specifying the `borrow` attribute.
-   Do an analysis of if this is a zero-overhead abstraction.  In particular, want to show that optimized code inlines everything to be equivalent to use of `String` and `str` (apart from calls to `validate`).
-   Update pneutype-derive crate to use latest of `darling`, `proc-macro2`, `quote`, and `syn` crates.
