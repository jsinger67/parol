MODULE case;


IMPORT Out, Modules;


BEGIN
    CASE Modules.ArgCount - 1 OF
        0   : Out.String("There are no arguments");
    |   1   : Out.String("There is one argument");
    |   2   : Out.String("There are two arguments");
    ELSE Out.String("There are more than two arguments") END;
    Out.Ln;
END case.
