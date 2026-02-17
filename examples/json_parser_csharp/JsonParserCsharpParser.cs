using System;
using System.Collections.Generic;
using Parol.Runtime;
using Parol.Runtime.Scanner;

namespace JsonParserCsharp {
    public class JsonParserCsharpParser {
    public static class JsonParserCsharpScannerData {
        public static readonly string[] TerminalNames = {
            "EndOfInput",
            "Newline",
            "Whitespace",
            "LineComment",
            "BlockComment",
            "LBrace",
            "RBrace",
            "Comma",
            "Colon",
            "LBracket",
            "RBracket",
            "True",
            "False",
            "Null",
            "String",
            "Number",
            "Error",
        };

        public static int? MatchFunction(char c) {
            var intervals = new (char Start, char End, int ClassIdx)[] {
                ('\0', '\u0008', 0),
                ('\t', '\t', 1),
                ('\n', '\n', 2),
                ('\u000b', '\u000c', 1),
                ('\r', '\r', 3),
                ('\u000e', '\u001f', 0),
                (' ', ' ', 1),
                ('!', '!', 0),
                ('"', '"', 4),
                ('#', '*', 0),
                ('+', '+', 5),
                (',', ',', 6),
                ('-', '-', 7),
                ('.', '.', 8),
                ('/', '/', 0),
                ('0', '0', 9),
                ('1', '9', 10),
                (':', ':', 11),
                (';', 'D', 0),
                ('E', 'E', 12),
                ('F', 'Z', 0),
                ('[', '[', 13),
                ('\\', '\\', 14),
                (']', ']', 15),
                ('^', '`', 0),
                ('a', 'a', 16),
                ('b', 'd', 0),
                ('e', 'e', 17),
                ('f', 'f', 18),
                ('g', 'k', 0),
                ('l', 'l', 19),
                ('m', 'm', 0),
                ('n', 'n', 20),
                ('o', 'q', 0),
                ('r', 'r', 21),
                ('s', 's', 22),
                ('t', 't', 23),
                ('u', 'u', 24),
                ('v', 'z', 0),
                ('{', '{', 25),
                ('|', '|', 0),
                ('}', '}', 26),
                ('~', '\u0084', 0),
                ('\u0085', '\u0085', 1),
                ('\u0086', '\u009f', 0),
                ('\u00a0', '\u00a0', 1),
                ('\u00a1', '\u167f', 0),
                ('\u1680', '\u1680', 1),
                ('\u1681', '\u1fff', 0),
                ('\u2000', '\u200a', 1),
                ('\u200b', '\u2027', 0),
                ('\u2028', '\u2029', 1),
                ('\u202a', '\u202e', 0),
                ('\u202f', '\u202f', 1),
                ('\u2030', '\u205e', 0),
                ('\u205f', '\u205f', 1),
                ('\u2060', '\u2fff', 0),
                ('\u3000', '\u3000', 1),
                ('\u3001', '\ufffe', 0),
            };

            int low = 0, high = intervals.Length - 1;
            while (low <= high) {
                int mid = low + (high - low) / 2;
                if (c >= intervals[mid].Start && c <= intervals[mid].End) return intervals[mid].ClassIdx;
                if (c < intervals[mid].Start) high = mid - 1;
                else low = mid + 1;
            }
            return null;
        }
        public static readonly ScannerMode[] ScannerModes = {
            new ScannerMode(
                "INITIAL",
                new Transition[] {
                },
                new Dfa(new DfaState[] {
                    new DfaState(
                        new DfaTransition?[] { new DfaTransition(18), new DfaTransition(25), new DfaTransition(29), new DfaTransition(30), new DfaTransition(19), new DfaTransition(18), new DfaTransition(13), new DfaTransition(20), new DfaTransition(18), new DfaTransition(15), new DfaTransition(14), new DfaTransition(24), new DfaTransition(18), new DfaTransition(27), new DfaTransition(18), new DfaTransition(28), new DfaTransition(18), new DfaTransition(18), new DfaTransition(21), new DfaTransition(18), new DfaTransition(23), new DfaTransition(18), new DfaTransition(18), new DfaTransition(22), new DfaTransition(18), new DfaTransition(31), new DfaTransition(32) },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(26), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(2), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1) },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { new DfaTransition(1), new DfaTransition(1), null, new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1) },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, new DfaTransition(4), null, new DfaTransition(4), null, new DfaTransition(16), new DfaTransition(16), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, new DfaTransition(16), new DfaTransition(16), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, new DfaTransition(17), new DfaTransition(17), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(33), null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(35), null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(10), null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(11), null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(34), null, null, null, null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(6), null, null, null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(7), null, null },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(7, 4, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, new DfaTransition(5), new DfaTransition(14), new DfaTransition(14), null, new DfaTransition(3), null, null, null, null, new DfaTransition(3), null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(15, 12, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, new DfaTransition(5), null, null, null, new DfaTransition(3), null, null, null, null, new DfaTransition(3), null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(15, 12, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, new DfaTransition(16), new DfaTransition(16), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(15, 12, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, new DfaTransition(17), new DfaTransition(17), null, new DfaTransition(3), null, null, null, null, new DfaTransition(3), null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(15, 12, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(16, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(26), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(2), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1), new DfaTransition(1) },
                        new AcceptData[] {
                            new AcceptData(16, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, new DfaTransition(15), new DfaTransition(14), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(16, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(9), null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(16, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(12), null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(16, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(8), null, null },
                        new AcceptData[] {
                            new AcceptData(16, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(8, 5, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, new DfaTransition(25), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(2, 1, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(14, 11, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(9, 6, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(10, 7, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(1, 0, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, new DfaTransition(29), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(1, 0, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(5, 2, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(6, 3, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(12, 9, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(13, 10, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(11, 8, new Lookahead.None()),
                        }
                    ),
                })
            ),
        };
    }


        public const int MaxK = 1;

        public static readonly string[] NonTerminalNames = {
            "Array",
            "ArrayList",
            "ArraySuffix",
            "Json",
            "Number",
            "Object",
            "ObjectList",
            "ObjectSuffix",
            "Pair",
            "String",
            "Value",
        };

        public static readonly LookaheadDfa[] LookaheadAutomata = {
            /* 0 - "Array" */
            new LookaheadDfa(
                7,
                new Trans[] {
                },
                0 // k
            ),
            /* 1 - "ArrayList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 7, 1, 10),
                    new Trans(0, 10, 2, 11),
                },
                1 // k
            ),
            /* 2 - "ArraySuffix" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 1, 8),
                    new Trans(0, 9, 1, 8),
                    new Trans(0, 10, 2, 9),
                    new Trans(0, 11, 1, 8),
                    new Trans(0, 12, 1, 8),
                    new Trans(0, 13, 1, 8),
                    new Trans(0, 14, 1, 8),
                    new Trans(0, 15, 1, 8),
                },
                1 // k
            ),
            /* 3 - "Json" */
            new LookaheadDfa(
                0,
                new Trans[] {
                },
                0 // k
            ),
            /* 4 - "Number" */
            new LookaheadDfa(
                20,
                new Trans[] {
                },
                0 // k
            ),
            /* 5 - "Object" */
            new LookaheadDfa(
                1,
                new Trans[] {
                },
                0 // k
            ),
            /* 6 - "ObjectList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 6, 2, 5),
                    new Trans(0, 7, 1, 4),
                },
                1 // k
            ),
            /* 7 - "ObjectSuffix" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 6, 2, 3),
                    new Trans(0, 14, 1, 2),
                },
                1 // k
            ),
            /* 8 - "Pair" */
            new LookaheadDfa(
                6,
                new Trans[] {
                },
                0 // k
            ),
            /* 9 - "String" */
            new LookaheadDfa(
                19,
                new Trans[] {
                },
                0 // k
            ),
            /* 10 - "Value" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 3, 14),
                    new Trans(0, 9, 4, 15),
                    new Trans(0, 11, 5, 16),
                    new Trans(0, 12, 6, 17),
                    new Trans(0, 13, 7, 18),
                    new Trans(0, 14, 1, 12),
                    new Trans(0, 15, 2, 13),
                },
                1 // k
            ),
        };

        public static readonly Production[] Productions = {
            // 0 - Json: Value;
            new Production(
                3,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 10),
                }
            ),
            // 1 - Object: '{'^ /* Clipped */ ObjectSuffix;
            new Production(
                5,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 5),
                    new ParseItem(ParseType.N, 7),
                }
            ),
            // 2 - ObjectSuffix: Pair ObjectList /* Vec */ '}'^ /* Clipped */;
            new Production(
                7,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 8),
                    new ParseItem(ParseType.N, 6),
                    new ParseItem(ParseType.C, 6),
                }
            ),
            // 3 - ObjectSuffix: '}'^ /* Clipped */;
            new Production(
                7,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 6),
                }
            ),
            // 4 - ObjectList: ','^ /* Clipped */ Pair ObjectList;
            new Production(
                6,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 7),
                    new ParseItem(ParseType.N, 8),
                    new ParseItem(ParseType.N, 6),
                }
            ),
            // 5 - ObjectList: ;
            new Production(
                6,
                new ParseItem[] {
                }
            ),
            // 6 - Pair: String ':'^ /* Clipped */ Value;
            new Production(
                8,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 9),
                    new ParseItem(ParseType.C, 8),
                    new ParseItem(ParseType.N, 10),
                }
            ),
            // 7 - Array: '['^ /* Clipped */ ArraySuffix;
            new Production(
                0,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 9),
                    new ParseItem(ParseType.N, 2),
                }
            ),
            // 8 - ArraySuffix: Value ArrayList /* Vec */ ']'^ /* Clipped */;
            new Production(
                2,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 10),
                    new ParseItem(ParseType.N, 1),
                    new ParseItem(ParseType.C, 10),
                }
            ),
            // 9 - ArraySuffix: ']'^ /* Clipped */;
            new Production(
                2,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 10),
                }
            ),
            // 10 - ArrayList: ','^ /* Clipped */ Value ArrayList;
            new Production(
                1,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 7),
                    new ParseItem(ParseType.N, 10),
                    new ParseItem(ParseType.N, 1),
                }
            ),
            // 11 - ArrayList: ;
            new Production(
                1,
                new ParseItem[] {
                }
            ),
            // 12 - Value: String;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 9),
                }
            ),
            // 13 - Value: Number;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 4),
                }
            ),
            // 14 - Value: Object;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 5),
                }
            ),
            // 15 - Value: Array;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 0),
                }
            ),
            // 16 - Value: 'true'^ /* Clipped */;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 11),
                }
            ),
            // 17 - Value: 'false'^ /* Clipped */;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 12),
                }
            ),
            // 18 - Value: 'null'^ /* Clipped */;
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 13),
                }
            ),
            // 19 - String: /"(\\.|[^"\\])*"/;
            new Production(
                9,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 14),
                }
            ),
            // 20 - Number: /-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?[0-9]+)?/;
            new Production(
                4,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 15),
                }
            ),
        };

        public static void Parse(string input, string fileName, IJsonParserCsharpActions userActions) {
            ParseInternal(input, fileName, userActions);
        }

        public static void Parse(string input, string fileName, IUserActions userActions) {
            ParseInternal(input, fileName, userActions);
        }

        private static void ParseInternal(string input, string fileName, IUserActions userActions) {
            var parser = new LLKParser(
                3,
                LookaheadAutomata,
                Productions,
                JsonParserCsharpScannerData.TerminalNames,
                NonTerminalNames
            );

            var tokens = Scanner.Scan(input, fileName, JsonParserCsharpScannerData.MatchFunction, JsonParserCsharpScannerData.ScannerModes);
            parser.Parse(tokens, userActions);
        }
    }
}
