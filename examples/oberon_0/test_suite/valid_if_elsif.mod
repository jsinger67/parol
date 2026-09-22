MODULE ValidIfElsif;
    VAR x: INTEGER;
BEGIN
    IF x < 0 THEN x := 0
    ELSIF x = 1 THEN x := 2
    ELSE x := 3
    END
END ValidIfElsif.
