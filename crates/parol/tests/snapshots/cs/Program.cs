using System;
using System.IO;
using SnapshotCs;

namespace SnapshotCs
{
    class Program
    {
        static void Main(string[] args)
        {
            if (args.Length < 1)
            {
                Console.WriteLine("Please provide a file name as first parameter!");
                return;
            }

            string fileName = args[0];
            string input = File.ReadAllText(fileName);
            ISnapshotCsActions actions = new SnapshotCsActions();

            try
            {
                SnapshotCsParser.Parse(input, fileName, actions);
                Console.WriteLine("Success!");
                Console.WriteLine(actions.ToString());
            }
            catch (Exception e)
            {
                Console.WriteLine($"Error: {e.Message}");
            }
        }
    }
}
