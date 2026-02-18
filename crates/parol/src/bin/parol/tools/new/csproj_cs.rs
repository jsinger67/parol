use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub(crate) struct CsProjCsData<'a> {
    pub(crate) _crate_name: &'a str,
}

impl std::fmt::Display for CsProjCsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Parol.Runtime" Version="0.1.1" />
  </ItemGroup>

  <Import Project="parol.targets" />

</Project>
"#
        )
    }
}
