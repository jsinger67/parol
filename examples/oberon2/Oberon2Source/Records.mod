MODULE record;


IMPORT Out;


CONST
    Male    = 1;
    Female  = 2;


TYPE
    SexDesc = INTEGER;
    Person  = RECORD
        Name    : ARRAY 32 OF CHAR;
        Age     : INTEGER;
        Title   : ARRAY 64 OF CHAR;
        Sex     : SexDesc;
    END;


VAR
    i           : INTEGER;
    employer    : Person;
    employee    : ARRAY 2 OF Person;


PROCEDURE dumpPerson ( p : Person );
BEGIN
    Out.String("Meet "); Out.String(p.Name);
    IF p.Sex = Male     THEN Out.String(". He is ")     END;
    IF p.Sex = Female   THEN Out.String(". She is ")    END;
    Out.Int(p.Age, 0); Out.String(" years old and a "); Out.String(p.Title); Out.Ln;
END dumpPerson;



BEGIN
    (* define people *)
    employer.Name := "Bing"; employer.Age := 42; employer.Title := "CEO"; employer.Sex := Male;

    employee[0].Name := "Bob"; employee[0].Age := 26;
    employee[0].Title := "SysAdmin"; employee[0].Sex := Male;

    employee[1].Name := "Alice"  ; employee[1].Age := 22;
    employee[1].Title := "Programmer"; employee[1].Sex := Female;

    (* print people *)
    dumpPerson(employer);
    FOR i := 0 TO LEN(employee) - 1 DO
        dumpPerson(employee[i]);
    END;
END record.
