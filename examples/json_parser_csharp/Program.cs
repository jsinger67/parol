using System;
using System.Diagnostics;
using System.IO;
using System.Linq;

namespace JsonParserCsharp
{
    internal static class Program
    {
        private static void Main(string[] args)
        {
            Stopwatch stopwatch = Stopwatch.StartNew();

            try
            {
                if (args.Length < 1)
                {
                    Console.WriteLine("Please provide a file name as first parameter!");
                    Console.WriteLine("Usage: JsonParserCsharp <fileName> [iterations]");
                    return;
                }

                string fileName = args[0];
                string input = File.ReadAllText(fileName);
                int iterations = 1;
                if (args.Length >= 2 && (!int.TryParse(args[1], out iterations) || iterations < 1))
                {
                    Console.WriteLine("Optional second parameter 'iterations' must be an integer greater than 0.");
                    return;
                }

                double[] parseDurationsMs = new double[iterations];
                IJsonParserCsharpActions? actions = null;

                for (int i = 0; i < iterations; i++)
                {
                    actions = new JsonRenderActions();
                    Stopwatch parseWatch = Stopwatch.StartNew();

                    try
                    {
                        JsonParserCsharpParser.Parse(input, fileName, actions);
                    }
                    catch (Exception e)
                    {
                        Console.WriteLine($"Error in iteration {i + 1}: {e}");
                        return;
                    }
                    finally
                    {
                        parseWatch.Stop();
                        parseDurationsMs[i] = parseWatch.Elapsed.TotalMilliseconds;
                    }
                }

                Console.WriteLine("Success!");
                if (iterations == 1 && actions != null)
                {
                    Console.WriteLine(actions.ToString());
                }
                else
                {
                    double min = parseDurationsMs.Min();
                    double max = parseDurationsMs.Max();
                    double avg = parseDurationsMs.Average();
                    double median = Median(parseDurationsMs);
                    Console.WriteLine(
                        $"Parse duration summary ({iterations} runs): min {min:F3} ms, median {median:F3} ms, avg {avg:F3} ms, max {max:F3} ms.");
                }
            }
            finally
            {
                stopwatch.Stop();
                Console.WriteLine($"Duration summary: total runtime {stopwatch.Elapsed.TotalMilliseconds:F3} ms ({stopwatch.Elapsed}).");
            }
        }

        private static double Median(double[] values)
        {
            double[] sorted = values.OrderBy(v => v).ToArray();
            int middle = sorted.Length / 2;

            if (sorted.Length % 2 == 0)
            {
                return (sorted[middle - 1] + sorted[middle]) / 2.0;
            }

            return sorted[middle];
        }
    }
}
