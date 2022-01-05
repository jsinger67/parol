# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## v0.5.4 - 2022-01-05

* Fixed a serious bug in parsing groups, repetitions and optionals introduced in commit [6476e75].
* Started issuing more detailed miette-like errors from parol itself.
* Fixed an invalid generation of the %pop() instruction form '%pop' to '%pop()'.
* More tests to check the parol parser's internal representation.
* Fixed some problems related to platform specific newline characters.
* Fixed Issue #5 *Thanks a lot to Techcable*

## v0.5.3 - 2022-01-02

As of this version a detailed changelog is maintained to help people to keep track of changes that have been made since last version of `parol`.

### Generation of sentences

An new tool (subcommand) `generate` was added to `parol` to generate an arbitrary sentence of a given grammar.
You can use it this way:

```shell
    >parol generate ./examples/json/json-exp.par
{ "\r" : "uA7Fcu8a4A񚥚\r" , "\b\f\nuD1C0u5daf\b" : null , "\n\/\f𘃈򘱵" : true , "\\󸽿\\\\uCfC4𚍑𞱁uD852" : "\b\buEA01\\" } 
```

I already found some quirks in a few regular expressions 😉.

Also you can run endless stress tests like in this example using a powershell one-liner:

```powershell
for (;;) { parol generate ./examples/json/json-exp.par | Set-Content "$env:Temp/x.json"; json_parser "$env:Temp/x.json"; if (-not $?) { break } }
```

#### Acknowledge

This was possible with the help of the awesome [rand_regex](https://github.com/kennytm/rand_regex.git) crate.

#### Disclaimer

On complex grammars the generation can get into deeply branching the grammar productions again and again because productions are randomly selected. Therefore generation is aborted with an error if the resulting sentence exceeds a certain limit. This limit currently defaults to a string length of 100 000. This value can be overwritten by giving an additional parameter after the grammar file.
If generation fails with error `parol::generators::language_generator::source_size_exceeded` please give it another try.
