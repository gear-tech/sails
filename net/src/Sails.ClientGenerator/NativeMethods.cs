namespace Sails.ClientGenerator;

internal static unsafe partial class NativeMethods
{
    private const string DllName = "libsails_net_client_gen";

    /// <summary>
    ///  # Safety
    ///
    ///  Function [`free_c_string`] should be called after this function
    /// </summary>
    internal delegate byte* GenerateDotnetClient(byte* program_utf8, int program_len, byte* config_utf8, int config_len);

    /// <summary>
    ///  # Safety
    ///
    ///  This function should not be called before the [`generate_dotnet_client`]
    /// </summary>
    internal delegate void FreeCString(byte* str);
}
