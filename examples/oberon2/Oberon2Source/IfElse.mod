MODULE ifelse;


IMPORT Out;

VAR n, m : INTEGER;


BEGIN
    n := 8; m := 4;

    IF n MOD m = 0 THEN
        Out.Int(n,0); Out.String(" is divisible by "); Out.Int(m,0); Out.Ln;
    END;

    n := 7; m := 6;

    IF n * m = 42 THEN
        Out.Int(n,0); Out.String(" times "); Out.Int(m,0); Out.String(" equals 42"); Out.Ln;
    END;

    IF n # m THEN Out.Int(n,0); Out.String(" does not equal "); Out.Int(m,0); Out.Ln; END;

    IF ODD(n)
        THEN
            Out.Int(n,0); Out.String(" is odd"); Out.Ln;
        ELSE
            Out.Int(n,0); Out.String(" is even"); Out.Ln;
    END;

    IF ~ODD(m)
        THEN
            Out.Int(m,0); Out.String(" is even"); Out.Ln;
        ELSE
            Out.Int(m,0); Out.String(" is odd"); Out.Ln;
    END;

    n := 9;

    IF n < 0
        THEN
            Out.Int(n, 0); Out.String(" is negative"); Out.Ln;
    ELSIF n < 10
        THEN
            Out.Int(n, 0); Out.String(" has 1 digit"); Out.Ln;
    ELSE
            Out.Int(n, 0); Out.String(" has multiple digits"); Out.Ln;
    END;
END ifelse.
