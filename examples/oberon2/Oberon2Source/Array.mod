MODULE arrays;

IMPORT Out;

VAR
    tmp     : INTEGER;
    matrix  : ARRAY 3 OF ARRAY 3 OF INTEGER;
    i, v, k : INTEGER;

BEGIN
    v := 1;
    FOR i := 0 TO LEN(matrix) - 1 DO
        FOR k := 0 TO LEN(matrix[i]) - 1 DO
            matrix[i][k] := v;
            INC(v);
        END;
    END;

    FOR i := 0 TO LEN(matrix) - 1 DO
        FOR k := 0 TO LEN(matrix[i]) - 1 DO
            Out.Int(matrix[i][k], 0); Out.String(" ");
        END;
        Out.Ln;
    END;

    FOR i := 0 TO LEN(matrix) - 1 DO
        FOR k := i + 1 TO LEN(matrix[i]) - 1 DO
            tmp := matrix[i][k];
            matrix[i][k] := matrix[k][i];
            matrix[k][i] := tmp;
        END;
    END;

    Out.Ln; Out.Ln;

    FOR i := 0 TO LEN(matrix) - 1 DO
        FOR k := 0 TO LEN(matrix[i]) - 1 DO
            Out.Int(matrix[i][k], 0); Out.String(" ");
        END;
        Out.Ln;
    END;
END arrays.
