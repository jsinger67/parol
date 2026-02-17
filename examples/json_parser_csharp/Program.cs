using System;
using System.IO;

namespace JsonParserCsharp
{
    internal static class Program
    {
        private static void Main(string[] args)
        {
            if (args.Length < 1)
            {
                Console.WriteLine("Please provide a file name as first parameter!");
                return;
            }

            string fileName = args[0];
            string input = File.ReadAllText(fileName);
            IJsonParserCsharpActions actions = new JsonRenderActions();

            try
            {
                JsonParserCsharpParser.Parse(input, fileName, actions);
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
