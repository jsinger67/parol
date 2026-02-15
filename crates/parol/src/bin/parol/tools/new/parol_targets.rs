use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub(crate) struct ParolTargetsData<'a> {
    pub(crate) crate_name: &'a str,
    pub(crate) grammar_name: String,
}

impl std::fmt::Display for ParolTargetsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParolTargetsData {
            crate_name,
            grammar_name,
        } = self;

        write!(
            f,
            r#"<Project xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  <Target Name="GenerateParser" BeforeTargets="BeforeCompile;CoreCompile" Inputs="{crate_name}.par" Outputs="{grammar_name}Parser.cs;I{grammar_name}Actions.cs">
    <Exec Command="parol -f {crate_name}.par -p {grammar_name}Parser.cs -a I{grammar_name}Actions.cs -t {grammar_name} -m {grammar_name} -l c-sharp" />
    <ItemGroup>
      <Compile Remove="{grammar_name}Parser.cs" />
      <Compile Include="{grammar_name}Parser.cs" />
      <Compile Remove="I{grammar_name}Actions.cs" />
      <Compile Include="I{grammar_name}Actions.cs" />
    </ItemGroup>
  </Target>
</Project>
"#
        )
    }
}
