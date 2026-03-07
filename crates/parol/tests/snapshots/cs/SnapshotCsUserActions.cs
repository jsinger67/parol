using System;
using Parol.Runtime;

namespace SnapshotCs
{
    // Extend the generated base actions with user-defined result handling.
    public partial class SnapshotCsUserActions : SnapshotCsActions
    {
        // Stores the start-symbol value so it can be easily used for grammar processing.
        private SnapshotCs? _parseResult;

        // Expose the parse result in a simple form for the scaffolded Program output.
        public override string ToString() => _parseResult?.ToString() ?? string.Empty;

        // Called when the start symbol has been parsed. Contains the processed input.
        public override void OnSnapshotCs(SnapshotCs arg)
        {
            _parseResult = arg;
        }
    }
}
