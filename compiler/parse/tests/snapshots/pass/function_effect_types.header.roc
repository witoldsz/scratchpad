platform "cli"
    requires {}{ main : Task {} [] } # TODO FIXME
    exposes []
    packages {}
    imports [ Task.{ Task } ]
    provides [ mainForHost ]
