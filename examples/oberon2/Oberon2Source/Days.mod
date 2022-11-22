MODULE Days;

  TYPE
    Day* = POINTER TO DayDesc;
    DayDesc = RECORD
      num: INTEGER
    END;
    Week* = ARRAY 7 OF Day;
  VAR
    sun*, mon*, tue*, wed*, thu*, fri*, sat* : Day;
    week: Week;

  PROCEDURE Next*(d : Day): Day;
  BEGIN RETURN week[(d.num + 1) MOD 7];
  END Next;

  PROCEDURE Prev*(d: Day): Day;
  BEGIN RETURN week[(d.num - 1) MOD 7];
  END Prev;

  PROCEDURE day(VAR d: Day; num: INTEGER);
  BEGIN NEW(d); d.num := num; week[num] := d;
  END day;

BEGIN
  day(sun, 0);
  day(mon, 1);
  day(tue, 2);
  day(wed, 3);
  day(thu, 4);
  day(fri, 5);
  day(sat, 6);
END Days.
