using System;
using System.IO;
using CalcCsharp;

namespace CalcCsharp
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
            ICalcCsharpActions actions = new CalcEvaluatorActions();

            try
            {
                CalcCsharpParser.Parse(input, fileName, actions);
                Console.WriteLine("Success!");
                Console.WriteLine(actions.ToString());
            }
            catch (Exception e)
            {
                Console.WriteLine($"Error: {e}");
            }
        }
    }
}
