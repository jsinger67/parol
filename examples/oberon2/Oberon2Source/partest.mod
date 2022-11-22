MODULE partest;

IMPORT Oberon, Texts;

CONST 
  argStr0 = "str"; (* we only have two types of args, one string and one int *)
  argInt0 = "int"; (* i. e. -str somestring -int somenumber *)

VAR
   W: Texts.Writer; (* for console output *)
   S: Texts.Scanner; T: Texts.Text;
BEGIN
    Texts.OpenWriter(W);
    Texts.WriteString(W, "hello, world, let's see which arguments do we get"); Texts.WriteLn(W);

    (* open arguments scanner *)
    Texts.OpenScanner(S, Oberon.Par.text, Oberon.Par.pos); 
    
    WHILE ~S.eot DO
       Texts.Scan(S);

       IF S.class = Texts.Char THEN (* do we get '-' sign ? *)
         IF S.c = "-" THEN
           Texts.Scan(S);
           IF S.class = Texts.Name THEN (* we got the key *)
             Texts.WriteString(W, "key: "); Texts.WriteString(W, S.s); Texts.WriteLn(W);
             (* now get the value *)
             IF S.s = argStr0 THEN
                Texts.Scan(S);
                IF S.class = Texts.Name THEN
                  Texts.WriteString(W, "value: "); Texts.WriteString (W, S.s); Texts.WriteLn(W); Texts.Append(Oberon.Log, W.buf);
                ELSE
                  Texts.WriteString(W, "string expected"); Texts.WriteLn(W);
                  Texts.Append(Oberon.Log, W.buf);
                  HALT(1);
                END;
             ELSIF S.s = argInt0 THEN
                Texts.Scan(S);
                IF S.class = Texts.Int THEN
                  Texts.WriteString(W, "value: "); Texts.WriteInt (W, S.i, 0); Texts.WriteLn(W); Texts.Append(Oberon.Log, W.buf);
                ELSE
                  Texts.WriteString(W, "integer expected"); Texts.WriteLn(W);
                  Texts.Append(Oberon.Log, W.buf);
                  HALT(1);
                END;
             END;
           ELSE
             (* we were expecting characters after the '-' sign *)
             Texts.WriteString(W, "key name expected"); Texts. WriteLn(W);
             Texts.Append(Oberon.Log, W.buf);
             HALT(1);
           END;
         END
       ELSE
         Texts.WriteString(W, "key option must start with '-' sign "); Texts.WriteLn(W);
       HALT(1);
       END; (* if got '-' *)
    Oberon.Par.pos := Texts.Pos(S);
    Texts.Append(Oberon.Log, W.buf)
    END; (* while *)


END partest.
