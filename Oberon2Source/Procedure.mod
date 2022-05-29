MODULE proc;


IMPORT Out;


PROCEDURE printSum(a, b : INTEGER);
BEGIN
    Out.Int(a + b, 0); Out.Ln
END printSum;


BEGIN
    printSum(6, 9)
END proc.
