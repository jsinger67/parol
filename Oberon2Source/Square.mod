MODULE square;


IMPORT Out;


VAR s : INTEGER;


PROCEDURE squared(x : INTEGER): INTEGER;
BEGIN
    RETURN x * x
END squared;


BEGIN
    s := squared(7);
    Out.Int(s, 0); Out.Ln;
    Out.Int(squared(8), 0); Out.Ln;
END square.
