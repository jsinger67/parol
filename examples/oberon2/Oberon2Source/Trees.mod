(* To run the test please issue:
   cargo run --bin oberon2 -- ./test.txt *)

MODULE Trees;  (* exports: Tree, Node, Insert, Search, Write, Init *)
  IMPORT Texts, Oberon;  (* exports read-only: Node.name *)

  TYPE
    Tree* = POINTER TO Node;
    Node* = RECORD
      name-: POINTER TO ARRAY OF CHAR;
      left, right: Tree
    END;

  VAR w: Texts.Writer;

  PROCEDURE (t: Tree) Insert* (name: ARRAY OF CHAR);
    VAR p, father: Tree;
  BEGIN p := t;
    REPEAT father := p;
      IF name = p.name^ THEN RETURN END;
      IF name < p.name^ THEN p := p.left ELSE p := p.right END
    UNTIL p = NIL;
    NEW(p); p.left := NIL; p.right := NIL;
    NEW(p.name, LEN(name)+1); COPY(name, p.name^);
    IF name < father.name^ THEN
      father.left := p
    ELSE
      father.right := p
    END
  END Insert;

  PROCEDURE (t: Tree) Search* (name: ARRAY OF CHAR): Tree;
    VAR p: Tree;
  BEGIN p := t;
    WHILE (p # NIL) & (name # p.name^) DO
      IF name < p.name^ THEN p := p.left ELSE p := p.right END
    END;
    RETURN p
  END Search;

  PROCEDURE (t: Tree) Write*;
  BEGIN
    IF t.left # NIL THEN t.left.Write END;
    Texts.WriteString(w, t.name^); Texts.WriteLn(w);
    Texts.Append(Oberon.Log, w.buf);
    IF t.right # NIL THEN t.right.Write END
  END Write;

  PROCEDURE Init* (t: Tree);
  BEGIN NEW(t.name, 1); t.name[0] := 0X; t.left := NIL; t.right := NIL
  END Init;

BEGIN Texts.OpenWriter(w)
END Trees.

(* End *)
