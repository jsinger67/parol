use miette::{bail, IntoDiagnostic, Result, WrapErr};
use parol::analysis::{decidable, explain_conflicts, FirstCache, FollowCache};
use parol::generators::generate_terminal_names;
use parol::obtain_grammar_config;
use parol::MAX_K;

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("decidable")
        .about("Can be used to detect the maximum lookahead needed for your grammar.")
        .arg(
            clap::Arg::with_name("grammar_file")
                .required(true)
                .short("f")
                .long("grammar-file")
                .takes_value(true)
                .help("The grammar file to use")
        )
        .arg(
            clap::Arg::with_name("lookahead")
                .short("k")
                .long("lookahead")
                .takes_value(true)
                .default_value("5")
                .help("The maximum number of lookahead tokens to be used")
        )
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .unwrap();

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    let max_k = args
        .value_of("lookahead")
        .unwrap()
        .parse::<usize>()
        .into_diagnostic()
        .wrap_err("Provide a valid integer value for second argument")?;

    if max_k > MAX_K {
        bail!("Maximum lookahead is {}", MAX_K);
    }

    let first_cache = FirstCache::new();
    let follow_cache = FollowCache::new();

    let mut errors = 0;
    let mut non_terminals_with_conflicts = vec![];
    let actual_k = grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .map(|n| {
            (
                n.clone(),
                decidable(&grammar_config, n, max_k, &first_cache, &follow_cache),
            )
        })
        .map(|(n, r)| {
            if let Ok(r) = r {
                r
            } else {
                errors += 1;
                non_terminals_with_conflicts.push(n);
                0
            }
        })
        .max();
    if errors > 0 {
        let terminals = generate_terminal_names(&grammar_config);
        for nt in &non_terminals_with_conflicts {
            println!("Conflicts for non-terminal '{}':", nt);
            let conflicts =
                explain_conflicts(&grammar_config, nt, max_k, &first_cache, &follow_cache)?;
            for (p1, t1, p2, t2) in conflicts {
                println!("  Conflict in productions {} and {}:", p1, p2);
                println!("    {}: {}", p1, t1.to_string(&terminals));
                println!("    {}: {}", p2, t2.to_string(&terminals));
                let intersection = t1.intersection(&t2);
                println!("    ∩: {}\n", intersection.to_string(&terminals));
            }
        }
        println!("{} undecidable non-terminal(s)", errors);
    } else {
        println!("Grammar is LL{}", actual_k.unwrap());
    }
    Ok(())
}
