MODULE varparam;


IMPORT Out;


VAR a, b : INTEGER;


PROCEDURE swapVals(VAR x, y : INTEGER);
VAR tmp : INTEGER;
BEGIN
    tmp := x; x := y; y := tmp;
END swapVals;


BEGIN
    a := 6; b := 9;
    Out.String("initial "); Out.Ln;
    Out.String("a : "); Out.Int(a, 0); Out.String("; b : "); Out.Int(b, 0); Out.Ln;
    swapVals(a, b);
    Out.String("after swap"); Out.Ln;
    Out.String("a : "); Out.Int(a, 0); Out.String("; b : "); Out.Int(b, 0); Out.Ln;
END varparam.
