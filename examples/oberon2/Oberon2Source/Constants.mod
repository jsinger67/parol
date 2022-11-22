MODULE constants;

IMPORT Out;

CONST
    s = "if it moves, compile it!";
    n = 42;
    m = n * 2;

BEGIN
    Out.String(s); Out.Ln;
    Out.Int(n, 0); Out.Ln;
    Out.Int(m, 0); Out.Ln;
END constants.
