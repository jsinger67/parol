MODULE gcd;


IMPORT Out, Modules;


VAR
    gcd         : INTEGER;
    arg0, arg1  : LONGINT;


PROCEDURE getGCD(a, b : INTEGER): INTEGER;
VAR ret : INTEGER;
BEGIN
    IF a = 0 THEN ret := b;
    ELSIF b = 0 THEN ret := a;
    ELSIF b > a THEN ret := getGCD(b, a);
    ELSE ret := getGCD(b, a MOD b) END;
    RETURN ret;
END getGCD;


BEGIN
    IF Modules.ArgCount # 3 THEN
        Out.String("enter two integers to get GCD"); Out.Ln;
        HALT(1)
    END;
    Modules.GetIntArg(1, arg0); Modules.GetIntArg(2, arg1);
    gcd := getGCD(SHORT(arg0), SHORT(arg1));
    Out.Int(gcd, 0); Out.Ln;
END gcd.
