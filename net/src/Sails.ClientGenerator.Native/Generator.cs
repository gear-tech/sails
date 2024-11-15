using static Sails.ClientGenerator.NativeMethods;

namespace Sails.ClientGenerator.Native;

public static class Generator
{
    public static unsafe string GenerateCode(string source, GeneratorConfig config)
    {
        using var library = LoadNativeLibrary();
        var generateFunc = library.LoadFunction<GenerateDotnetClient>("generate_dotnet_client");
        var freeFunc = library.LoadFunction<FreeCString>("free_c_string");

        var idlBytes = Encoding.UTF8.GetBytes(source);
        var configBytes = Encoding.UTF8.GetBytes(config.ToJsonString());

        fixed (byte* idlPtr = idlBytes)
        {
            fixed (byte* configPtr = configBytes)
            {
                var cstr = generateFunc(idlPtr, idlBytes.Length, configPtr, configBytes.Length);
                try
                {
                    return new string((sbyte*)cstr);
                }
                finally
                {
                    freeFunc(cstr);
                }
            }
        }
    }
}
