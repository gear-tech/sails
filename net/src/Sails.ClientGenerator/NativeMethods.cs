using System.Runtime.InteropServices;

namespace Sails.ClientGenerator;

internal static unsafe partial class NativeMethods
{
    private const string DllName = "sails_net_client_gen";

    /// <summary>
    ///  # Safety
    ///
    ///  Function [`free_c_string`] should be called after this function
    /// </summary>
    [DllImport(DllName, EntryPoint = "generate_dotnet_client", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
    internal static extern byte* generate_dotnet_client(byte* program_utf8, int program_len, byte* config_utf8, int config_len);

    /// <summary>
    ///  # Safety
    ///
    ///  This function should not be called before the [`generate_dotnet_client`]
    /// </summary>
    [DllImport(DllName, EntryPoint = "free_c_string", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
    internal static extern void free_c_string(byte* str);
}
