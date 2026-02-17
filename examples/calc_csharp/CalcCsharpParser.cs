using System;
using System.Collections.Generic;
using Parol.Runtime;
using Parol.Runtime.Scanner;

namespace CalcCsharp {
    public class CalcCsharpParser {
    public static class CalcCsharpScannerData {
        public static readonly string[] TerminalNames = {
            "EndOfInput",
            "Newline",
            "Whitespace",
            "LineComment",
            "BlockComment",
            "Semicolon",
            "EqualityOp",
            "AssignOp",
            "LogicalOrOp",
            "LogicalAndOp",
            "BitwiseOrOp",
            "BitwiseAndOp",
            "BitwiseShiftOp",
            "RelationalOp",
            "Plus",
            "Minus",
            "PowOp",
            "MultOp",
            "LParen",
            "RParen",
            "Number",
            "Id",
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
                ('!', '!', 4),
                ('"', '$', 0),
                ('%', '%', 5),
                ('&', '&', 6),
                ('\'', '\'', 0),
                ('(', '(', 7),
                (')', ')', 8),
                ('*', '*', 9),
                ('+', '+', 10),
                (',', ',', 0),
                ('-', '-', 11),
                ('.', '.', 0),
                ('/', '/', 12),
                ('0', '0', 13),
                ('1', '9', 14),
                (':', ':', 0),
                (';', ';', 15),
                ('<', '<', 16),
                ('=', '=', 17),
                ('>', '>', 18),
                ('?', '@', 0),
                ('A', 'Z', 19),
                ('[', ']', 0),
                ('^', '^', 20),
                ('_', '_', 19),
                ('`', '`', 0),
                ('a', 'z', 19),
                ('{', '{', 0),
                ('|', '|', 21),
                ('}', '\u0084', 0),
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
                        new DfaTransition?[] { new DfaTransition(6), new DfaTransition(19), new DfaTransition(28), new DfaTransition(29), new DfaTransition(8), new DfaTransition(26), new DfaTransition(1), new DfaTransition(21), new DfaTransition(22), new DfaTransition(24), new DfaTransition(27), new DfaTransition(5), new DfaTransition(25), new DfaTransition(11), new DfaTransition(12), new DfaTransition(13), new DfaTransition(16), new DfaTransition(4), new DfaTransition(17), new DfaTransition(20), new DfaTransition(7), new DfaTransition(30) },
                        new AcceptData[] {
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, new DfaTransition(2), null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(11, 9, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(9, 7, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(7, 5, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(18), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(7, 5, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(15, 13, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(22, 20, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(22, 20, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(18), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(22, 20, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(3, 2, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { new DfaTransition(10), new DfaTransition(10), new DfaTransition(9), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10), new DfaTransition(10) },
                        new AcceptData[] {
                            new AcceptData(3, 2, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(20, 18, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(12), new DfaTransition(12), null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(20, 18, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(5, 3, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(12, 10, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(13, 11, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(14), new DfaTransition(15), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(13, 11, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(15), new DfaTransition(14), null, null, null },
                        new AcceptData[] {
                            new AcceptData(13, 11, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(6, 4, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, new DfaTransition(19), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(2, 1, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(20), new DfaTransition(20), null, null, null, null, new DfaTransition(20), null, null },
                        new AcceptData[] {
                            new AcceptData(21, 19, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(18, 16, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(19, 17, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(16, 14, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, new DfaTransition(23), null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(17, 15, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(10), null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(17, 15, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(17, 15, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(14, 12, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(1, 0, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, new DfaTransition(28), null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(1, 0, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, new DfaTransition(3), null, null, null, new DfaTransition(31) },
                        new AcceptData[] {
                            new AcceptData(10, 8, new Lookahead.None()),
                        }
                    ),
                    new DfaState(
                        new DfaTransition?[] { null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null },
                        new AcceptData[] {
                            new AcceptData(8, 6, new Lookahead.None()),
                        }
                    ),
                })
            ),
        };
    }


        public const int MaxK = 2;

        public static readonly string[] NonTerminalNames = {
            "AddOp",
            "AssignItem",
            "AssignOp",
            "Assignment",
            "AssignmentList",
            "BitwiseAnd",
            "BitwiseAndList",
            "BitwiseAndOp",
            "BitwiseOr",
            "BitwiseOrList",
            "BitwiseOrOp",
            "BitwiseShift",
            "BitwiseShiftList",
            "BitwiseShiftOp",
            "Calc",
            "CalcList",
            "Equality",
            "EqualityList",
            "EqualityOp",
            "Factor",
            "Id",
            "IdRef",
            "Instruction",
            "LogicalAnd",
            "LogicalAndList",
            "LogicalAndOp",
            "LogicalOr",
            "LogicalOrList",
            "LogicalOrOp",
            "Minus",
            "Mult",
            "MultList",
            "MultOp",
            "Negate",
            "Number",
            "Plus",
            "PowOp",
            "Power",
            "PowerList",
            "Relational",
            "RelationalList",
            "RelationalOp",
            "Summ",
            "SummList",
        };

        public static readonly LookaheadDfa[] LookaheadAutomata = {
            /* 0 - "AddOp" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 14, 1, 42),
                    new Trans(0, 15, 2, 43),
                },
                1 // k
            ),
            /* 1 - "AssignItem" */
            new LookaheadDfa(
                17,
                new Trans[] {
                },
                0 // k
            ),
            /* 2 - "AssignOp" */
            new LookaheadDfa(
                4,
                new Trans[] {
                },
                0 // k
            ),
            /* 3 - "Assignment" */
            new LookaheadDfa(
                18,
                new Trans[] {
                },
                0 // k
            ),
            /* 4 - "AssignmentList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 15, 3, -1),
                    new Trans(0, 18, 3, -1),
                    new Trans(0, 20, 4, -1),
                    new Trans(0, 21, 1, -1),
                    new Trans(1, 5, 5, 20),
                    new Trans(1, 6, 5, 20),
                    new Trans(1, 7, 2, 19),
                    new Trans(1, 8, 5, 20),
                    new Trans(1, 9, 5, 20),
                    new Trans(1, 10, 5, 20),
                    new Trans(1, 11, 5, 20),
                    new Trans(1, 12, 5, 20),
                    new Trans(1, 13, 5, 20),
                    new Trans(1, 14, 5, 20),
                    new Trans(1, 15, 5, 20),
                    new Trans(1, 16, 5, 20),
                    new Trans(1, 17, 5, 20),
                    new Trans(3, 15, 5, 20),
                    new Trans(3, 18, 5, 20),
                    new Trans(3, 20, 5, 20),
                    new Trans(3, 21, 5, 20),
                    new Trans(4, 5, 5, 20),
                    new Trans(4, 6, 5, 20),
                    new Trans(4, 8, 5, 20),
                    new Trans(4, 9, 5, 20),
                    new Trans(4, 10, 5, 20),
                    new Trans(4, 11, 5, 20),
                    new Trans(4, 12, 5, 20),
                    new Trans(4, 13, 5, 20),
                    new Trans(4, 14, 5, 20),
                    new Trans(4, 15, 5, 20),
                    new Trans(4, 16, 5, 20),
                    new Trans(4, 17, 5, 20),
                },
                2 // k
            ),
            /* 5 - "BitwiseAnd" */
            new LookaheadDfa(
                30,
                new Trans[] {
                },
                0 // k
            ),
            /* 6 - "BitwiseAndList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 32),
                    new Trans(0, 8, 2, 32),
                    new Trans(0, 9, 2, 32),
                    new Trans(0, 10, 2, 32),
                    new Trans(0, 11, 1, 31),
                    new Trans(0, 19, 2, 32),
                },
                1 // k
            ),
            /* 7 - "BitwiseAndOp" */
            new LookaheadDfa(
                8,
                new Trans[] {
                },
                0 // k
            ),
            /* 8 - "BitwiseOr" */
            new LookaheadDfa(
                27,
                new Trans[] {
                },
                0 // k
            ),
            /* 9 - "BitwiseOrList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 29),
                    new Trans(0, 8, 2, 29),
                    new Trans(0, 9, 2, 29),
                    new Trans(0, 10, 1, 28),
                    new Trans(0, 19, 2, 29),
                },
                1 // k
            ),
            /* 10 - "BitwiseOrOp" */
            new LookaheadDfa(
                7,
                new Trans[] {
                },
                0 // k
            ),
            /* 11 - "BitwiseShift" */
            new LookaheadDfa(
                39,
                new Trans[] {
                },
                0 // k
            ),
            /* 12 - "BitwiseShiftList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 41),
                    new Trans(0, 6, 2, 41),
                    new Trans(0, 8, 2, 41),
                    new Trans(0, 9, 2, 41),
                    new Trans(0, 10, 2, 41),
                    new Trans(0, 11, 2, 41),
                    new Trans(0, 12, 1, 40),
                    new Trans(0, 13, 2, 41),
                    new Trans(0, 19, 2, 41),
                },
                1 // k
            ),
            /* 13 - "BitwiseShiftOp" */
            new LookaheadDfa(
                9,
                new Trans[] {
                },
                0 // k
            ),
            /* 14 - "Calc" */
            new LookaheadDfa(
                0,
                new Trans[] {
                },
                0 // k
            ),
            /* 15 - "CalcList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 0, 2, 2),
                    new Trans(0, 15, 1, 1),
                    new Trans(0, 18, 1, 1),
                    new Trans(0, 20, 1, 1),
                    new Trans(0, 21, 1, 1),
                },
                1 // k
            ),
            /* 16 - "Equality" */
            new LookaheadDfa(
                33,
                new Trans[] {
                },
                0 // k
            ),
            /* 17 - "EqualityList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 35),
                    new Trans(0, 6, 1, 34),
                    new Trans(0, 8, 2, 35),
                    new Trans(0, 9, 2, 35),
                    new Trans(0, 10, 2, 35),
                    new Trans(0, 11, 2, 35),
                    new Trans(0, 19, 2, 35),
                },
                1 // k
            ),
            /* 18 - "EqualityOp" */
            new LookaheadDfa(
                3,
                new Trans[] {
                },
                0 // k
            ),
            /* 19 - "Factor" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 15, 3, 56),
                    new Trans(0, 18, 4, 57),
                    new Trans(0, 20, 1, 54),
                    new Trans(0, 21, 2, 55),
                },
                1 // k
            ),
            /* 20 - "Id" */
            new LookaheadDfa(
                60,
                new Trans[] {
                },
                0 // k
            ),
            /* 21 - "IdRef" */
            new LookaheadDfa(
                59,
                new Trans[] {
                },
                0 // k
            ),
            /* 22 - "Instruction" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 15, 3, -1),
                    new Trans(0, 18, 3, -1),
                    new Trans(0, 20, 4, -1),
                    new Trans(0, 21, 1, -1),
                    new Trans(1, 5, 5, 16),
                    new Trans(1, 6, 5, 16),
                    new Trans(1, 7, 2, 15),
                    new Trans(1, 8, 5, 16),
                    new Trans(1, 9, 5, 16),
                    new Trans(1, 10, 5, 16),
                    new Trans(1, 11, 5, 16),
                    new Trans(1, 12, 5, 16),
                    new Trans(1, 13, 5, 16),
                    new Trans(1, 14, 5, 16),
                    new Trans(1, 15, 5, 16),
                    new Trans(1, 16, 5, 16),
                    new Trans(1, 17, 5, 16),
                    new Trans(3, 15, 5, 16),
                    new Trans(3, 18, 5, 16),
                    new Trans(3, 20, 5, 16),
                    new Trans(3, 21, 5, 16),
                    new Trans(4, 5, 5, 16),
                    new Trans(4, 6, 5, 16),
                    new Trans(4, 8, 5, 16),
                    new Trans(4, 9, 5, 16),
                    new Trans(4, 10, 5, 16),
                    new Trans(4, 11, 5, 16),
                    new Trans(4, 12, 5, 16),
                    new Trans(4, 13, 5, 16),
                    new Trans(4, 14, 5, 16),
                    new Trans(4, 15, 5, 16),
                    new Trans(4, 16, 5, 16),
                    new Trans(4, 17, 5, 16),
                },
                2 // k
            ),
            /* 23 - "LogicalAnd" */
            new LookaheadDfa(
                24,
                new Trans[] {
                },
                0 // k
            ),
            /* 24 - "LogicalAndList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 26),
                    new Trans(0, 8, 2, 26),
                    new Trans(0, 9, 1, 25),
                    new Trans(0, 19, 2, 26),
                },
                1 // k
            ),
            /* 25 - "LogicalAndOp" */
            new LookaheadDfa(
                6,
                new Trans[] {
                },
                0 // k
            ),
            /* 26 - "LogicalOr" */
            new LookaheadDfa(
                21,
                new Trans[] {
                },
                0 // k
            ),
            /* 27 - "LogicalOrList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 23),
                    new Trans(0, 8, 1, 22),
                    new Trans(0, 19, 2, 23),
                },
                1 // k
            ),
            /* 28 - "LogicalOrOp" */
            new LookaheadDfa(
                5,
                new Trans[] {
                },
                0 // k
            ),
            /* 29 - "Minus" */
            new LookaheadDfa(
                12,
                new Trans[] {
                },
                0 // k
            ),
            /* 30 - "Mult" */
            new LookaheadDfa(
                47,
                new Trans[] {
                },
                0 // k
            ),
            /* 31 - "MultList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 49),
                    new Trans(0, 6, 2, 49),
                    new Trans(0, 8, 2, 49),
                    new Trans(0, 9, 2, 49),
                    new Trans(0, 10, 2, 49),
                    new Trans(0, 11, 2, 49),
                    new Trans(0, 12, 2, 49),
                    new Trans(0, 13, 2, 49),
                    new Trans(0, 14, 2, 49),
                    new Trans(0, 15, 2, 49),
                    new Trans(0, 17, 1, 48),
                    new Trans(0, 19, 2, 49),
                },
                1 // k
            ),
            /* 32 - "MultOp" */
            new LookaheadDfa(
                14,
                new Trans[] {
                },
                0 // k
            ),
            /* 33 - "Negate" */
            new LookaheadDfa(
                53,
                new Trans[] {
                },
                0 // k
            ),
            /* 34 - "Number" */
            new LookaheadDfa(
                58,
                new Trans[] {
                },
                0 // k
            ),
            /* 35 - "Plus" */
            new LookaheadDfa(
                11,
                new Trans[] {
                },
                0 // k
            ),
            /* 36 - "PowOp" */
            new LookaheadDfa(
                13,
                new Trans[] {
                },
                0 // k
            ),
            /* 37 - "Power" */
            new LookaheadDfa(
                50,
                new Trans[] {
                },
                0 // k
            ),
            /* 38 - "PowerList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 52),
                    new Trans(0, 6, 2, 52),
                    new Trans(0, 8, 2, 52),
                    new Trans(0, 9, 2, 52),
                    new Trans(0, 10, 2, 52),
                    new Trans(0, 11, 2, 52),
                    new Trans(0, 12, 2, 52),
                    new Trans(0, 13, 2, 52),
                    new Trans(0, 14, 2, 52),
                    new Trans(0, 15, 2, 52),
                    new Trans(0, 16, 1, 51),
                    new Trans(0, 17, 2, 52),
                    new Trans(0, 19, 2, 52),
                },
                1 // k
            ),
            /* 39 - "Relational" */
            new LookaheadDfa(
                36,
                new Trans[] {
                },
                0 // k
            ),
            /* 40 - "RelationalList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 38),
                    new Trans(0, 6, 2, 38),
                    new Trans(0, 8, 2, 38),
                    new Trans(0, 9, 2, 38),
                    new Trans(0, 10, 2, 38),
                    new Trans(0, 11, 2, 38),
                    new Trans(0, 13, 1, 37),
                    new Trans(0, 19, 2, 38),
                },
                1 // k
            ),
            /* 41 - "RelationalOp" */
            new LookaheadDfa(
                10,
                new Trans[] {
                },
                0 // k
            ),
            /* 42 - "Summ" */
            new LookaheadDfa(
                44,
                new Trans[] {
                },
                0 // k
            ),
            /* 43 - "SummList" */
            new LookaheadDfa(
                -1,
                new Trans[] {
                    new Trans(0, 5, 2, 46),
                    new Trans(0, 6, 2, 46),
                    new Trans(0, 8, 2, 46),
                    new Trans(0, 9, 2, 46),
                    new Trans(0, 10, 2, 46),
                    new Trans(0, 11, 2, 46),
                    new Trans(0, 12, 2, 46),
                    new Trans(0, 13, 2, 46),
                    new Trans(0, 14, 1, 45),
                    new Trans(0, 15, 1, 45),
                    new Trans(0, 19, 2, 46),
                },
                1 // k
            ),
        };

        public static readonly Production[] Productions = {
            // 0 - Calc: CalcList /* Vec */;
            new Production(
                14,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 15),
                }
            ),
            // 1 - CalcList: Instruction ";"^ /* Clipped */ CalcList;
            new Production(
                15,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 22),
                    new ParseItem(ParseType.C, 5),
                    new ParseItem(ParseType.N, 15),
                }
            ),
            // 2 - CalcList: ;
            new Production(
                15,
                new ParseItem[] {
                }
            ),
            // 3 - EqualityOp: "==|!=";
            new Production(
                18,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 6),
                }
            ),
            // 4 - AssignOp: "(\+|-|\*|/|%|<<|>>|&|\^|\|)?=";
            new Production(
                2,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 7),
                }
            ),
            // 5 - LogicalOrOp: "\|\|";
            new Production(
                28,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 8),
                }
            ),
            // 6 - LogicalAndOp: "&&";
            new Production(
                25,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 9),
                }
            ),
            // 7 - BitwiseOrOp: "\|";
            new Production(
                10,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 10),
                }
            ),
            // 8 - BitwiseAndOp: "&";
            new Production(
                7,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 11),
                }
            ),
            // 9 - BitwiseShiftOp: "<<|>>";
            new Production(
                13,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 12),
                }
            ),
            // 10 - RelationalOp: "<=|<|>=|>";
            new Production(
                41,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 13),
                }
            ),
            // 11 - Plus: "\+";
            new Production(
                35,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 14),
                }
            ),
            // 12 - Minus: "-";
            new Production(
                29,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 15),
                }
            ),
            // 13 - PowOp: "\*\*";
            new Production(
                36,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 16),
                }
            ),
            // 14 - MultOp: "\*|/|%";
            new Production(
                32,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 17),
                }
            ),
            // 15 - Instruction: Assignment;
            new Production(
                22,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 3),
                }
            ),
            // 16 - Instruction: LogicalOr;
            new Production(
                22,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 26),
                }
            ),
            // 17 - AssignItem: Id AssignOp;
            new Production(
                1,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 20),
                    new ParseItem(ParseType.N, 2),
                }
            ),
            // 18 - Assignment: AssignItem AssignmentList /* Vec */ LogicalOr;
            new Production(
                3,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 1),
                    new ParseItem(ParseType.N, 4),
                    new ParseItem(ParseType.N, 26),
                }
            ),
            // 19 - AssignmentList: AssignItem AssignmentList;
            new Production(
                4,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 1),
                    new ParseItem(ParseType.N, 4),
                }
            ),
            // 20 - AssignmentList: ;
            new Production(
                4,
                new ParseItem[] {
                }
            ),
            // 21 - LogicalOr: LogicalAnd LogicalOrList /* Vec */;
            new Production(
                26,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 23),
                    new ParseItem(ParseType.N, 27),
                }
            ),
            // 22 - LogicalOrList: LogicalOrOp LogicalAnd LogicalOrList;
            new Production(
                27,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 28),
                    new ParseItem(ParseType.N, 23),
                    new ParseItem(ParseType.N, 27),
                }
            ),
            // 23 - LogicalOrList: ;
            new Production(
                27,
                new ParseItem[] {
                }
            ),
            // 24 - LogicalAnd: BitwiseOr LogicalAndList /* Vec */;
            new Production(
                23,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 8),
                    new ParseItem(ParseType.N, 24),
                }
            ),
            // 25 - LogicalAndList: LogicalAndOp BitwiseOr LogicalAndList;
            new Production(
                24,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 25),
                    new ParseItem(ParseType.N, 8),
                    new ParseItem(ParseType.N, 24),
                }
            ),
            // 26 - LogicalAndList: ;
            new Production(
                24,
                new ParseItem[] {
                }
            ),
            // 27 - BitwiseOr: BitwiseAnd BitwiseOrList /* Vec */;
            new Production(
                8,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 5),
                    new ParseItem(ParseType.N, 9),
                }
            ),
            // 28 - BitwiseOrList: BitwiseOrOp BitwiseAnd BitwiseOrList;
            new Production(
                9,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 10),
                    new ParseItem(ParseType.N, 5),
                    new ParseItem(ParseType.N, 9),
                }
            ),
            // 29 - BitwiseOrList: ;
            new Production(
                9,
                new ParseItem[] {
                }
            ),
            // 30 - BitwiseAnd: Equality BitwiseAndList /* Vec */;
            new Production(
                5,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 16),
                    new ParseItem(ParseType.N, 6),
                }
            ),
            // 31 - BitwiseAndList: BitwiseAndOp Equality BitwiseAndList;
            new Production(
                6,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 7),
                    new ParseItem(ParseType.N, 16),
                    new ParseItem(ParseType.N, 6),
                }
            ),
            // 32 - BitwiseAndList: ;
            new Production(
                6,
                new ParseItem[] {
                }
            ),
            // 33 - Equality: Relational EqualityList /* Vec */;
            new Production(
                16,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 39),
                    new ParseItem(ParseType.N, 17),
                }
            ),
            // 34 - EqualityList: EqualityOp Relational EqualityList;
            new Production(
                17,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 18),
                    new ParseItem(ParseType.N, 39),
                    new ParseItem(ParseType.N, 17),
                }
            ),
            // 35 - EqualityList: ;
            new Production(
                17,
                new ParseItem[] {
                }
            ),
            // 36 - Relational: BitwiseShift RelationalList /* Vec */;
            new Production(
                39,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 11),
                    new ParseItem(ParseType.N, 40),
                }
            ),
            // 37 - RelationalList: RelationalOp BitwiseShift RelationalList;
            new Production(
                40,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 41),
                    new ParseItem(ParseType.N, 11),
                    new ParseItem(ParseType.N, 40),
                }
            ),
            // 38 - RelationalList: ;
            new Production(
                40,
                new ParseItem[] {
                }
            ),
            // 39 - BitwiseShift: Summ BitwiseShiftList /* Vec */;
            new Production(
                11,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 42),
                    new ParseItem(ParseType.N, 12),
                }
            ),
            // 40 - BitwiseShiftList: BitwiseShiftOp Summ BitwiseShiftList;
            new Production(
                12,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 13),
                    new ParseItem(ParseType.N, 42),
                    new ParseItem(ParseType.N, 12),
                }
            ),
            // 41 - BitwiseShiftList: ;
            new Production(
                12,
                new ParseItem[] {
                }
            ),
            // 42 - AddOp: Plus;
            new Production(
                0,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 35),
                }
            ),
            // 43 - AddOp: Minus;
            new Production(
                0,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 29),
                }
            ),
            // 44 - Summ: Mult SummList /* Vec */;
            new Production(
                42,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 30),
                    new ParseItem(ParseType.N, 43),
                }
            ),
            // 45 - SummList: AddOp Mult SummList;
            new Production(
                43,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 0),
                    new ParseItem(ParseType.N, 30),
                    new ParseItem(ParseType.N, 43),
                }
            ),
            // 46 - SummList: ;
            new Production(
                43,
                new ParseItem[] {
                }
            ),
            // 47 - Mult: Power MultList /* Vec */;
            new Production(
                30,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 37),
                    new ParseItem(ParseType.N, 31),
                }
            ),
            // 48 - MultList: MultOp Power MultList;
            new Production(
                31,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 32),
                    new ParseItem(ParseType.N, 37),
                    new ParseItem(ParseType.N, 31),
                }
            ),
            // 49 - MultList: ;
            new Production(
                31,
                new ParseItem[] {
                }
            ),
            // 50 - Power: Factor PowerList /* Vec */;
            new Production(
                37,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 19),
                    new ParseItem(ParseType.N, 38),
                }
            ),
            // 51 - PowerList: PowOp Factor PowerList;
            new Production(
                38,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 36),
                    new ParseItem(ParseType.N, 19),
                    new ParseItem(ParseType.N, 38),
                }
            ),
            // 52 - PowerList: ;
            new Production(
                38,
                new ParseItem[] {
                }
            ),
            // 53 - Negate: Minus;
            new Production(
                33,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 29),
                }
            ),
            // 54 - Factor: Number;
            new Production(
                19,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 34),
                }
            ),
            // 55 - Factor: IdRef;
            new Production(
                19,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 21),
                }
            ),
            // 56 - Factor: Negate Factor;
            new Production(
                19,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 33),
                    new ParseItem(ParseType.N, 19),
                }
            ),
            // 57 - Factor: "\("^ /* Clipped */ LogicalOr "\)"^ /* Clipped */;
            new Production(
                19,
                new ParseItem[] {
                    new ParseItem(ParseType.C, 18),
                    new ParseItem(ParseType.N, 26),
                    new ParseItem(ParseType.C, 19),
                }
            ),
            // 58 - Number: "0|[1-9][0-9]*";
            new Production(
                34,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 20),
                }
            ),
            // 59 - IdRef: Id;
            new Production(
                21,
                new ParseItem[] {
                    new ParseItem(ParseType.N, 20),
                }
            ),
            // 60 - Id: "[a-zA-Z_][a-zA-Z0-9_]*";
            new Production(
                20,
                new ParseItem[] {
                    new ParseItem(ParseType.T, 21),
                }
            ),
        };

        public static void Parse(string input, string fileName, ICalcCsharpActions userActions) {
            ParseInternal(input, fileName, userActions);
        }

        public static void Parse(string input, string fileName, IUserActions userActions) {
            ParseInternal(input, fileName, userActions);
        }

        private static void ParseInternal(string input, string fileName, IUserActions userActions) {
            var parser = new LLKParser(
                14,
                LookaheadAutomata,
                Productions,
                CalcCsharpScannerData.TerminalNames,
                NonTerminalNames
            );

            var tokens = Scanner.Scan(input, fileName, CalcCsharpScannerData.MatchFunction, CalcCsharpScannerData.ScannerModes);
            parser.Parse(tokens, userActions);
        }
    }
}
