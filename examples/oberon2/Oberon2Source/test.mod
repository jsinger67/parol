MODULE test;

IMPORT Days, Out;

VAR today, yesterday, tomorrow: Days.Day;

BEGIN
  today := Days.mon; (*init*)

  yesterday := Days.Prev(today);
  IF yesterday = Days.sun
  THEN
    Out.String("it works!"); Out.Ln
  END;
  tomorrow := Days.Next(today);

  IF tomorrow = Days.tue
  THEN
    Out.String("it works!"); Out.Ln
  END;

END test.
