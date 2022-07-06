platform "test-platform"
    requires {} { main : _ }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

# This case is important to test because there's no padding
# after the largest variant, so the compiler adds an extra u8
# (rounded up to alignment, so an an extra 8 bytes) in which
# to store the discriminant. We have to bindgen accordingly!
NonRecursive : [Foo Str, Bar I64, Blah I32, Baz]

mainForHost : NonRecursive
mainForHost = main
