Sails.ClientGenerator
=====================

A library generating .NET code for communicating with programs written using
[Sails](https://github.com/gear-tech/sails?tab=readme-ov-file#sails-) framework
based on their IDL files. The generated code relies on abstractions provided by
[Sails.Net](https://www.nuget.org/packages/Sails.Net).

### Usage

In the `csproj` file of the project willing to use generated client code, add:
```xml
<ItemGroup>
  <!-- IDL file obtained from Sails program -->
  <AdditionalFile Include="demo.idl" />
</ItemGroup>

<ItemGroup>
  <PackageReference Include="Sails.ClientGenerator">
      <PrivateAssets>all</PrivateAssets>
    </PackageReference>
  <PackageReference Include="Sails.Net" />
</ItemGroup>
```
This will generate .NET code for the client which will be found under
`Dependencies -> Analyzers -> Sails.ClientGenerator -> Sails.ClientGenerator.SailsClientGenerator`.
Discover how the generated code can be used via exploring
[tests](https://github.com/gear-tech/sails/tree/master/net/tests/Sails.DemoClient.Tests).
