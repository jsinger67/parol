MODULE ValidArrayRecord;
    TYPE T = RECORD
        a: INTEGER;
        b: ARRAY 10 OF INTEGER
    END;
    VAR r: T;
        i: INTEGER;
BEGIN
    r.a := 7;
    i := 0;
    WHILE i < 10 DO
        r.b[i] := i
    END
END ValidArrayRecord.
