#pragma warning disable RS1035 // Do not use APIs banned for analyzers

using System.Security.Cryptography;

namespace Sails.ClientGenerator;

internal static unsafe partial class NativeMethods
{
    internal static NativeLibrary LoadNativeLibrary()
    {
        var nativeLibraryPath = ExtractResourceToFile(DllName);
        return new NativeLibrary(nativeLibraryPath);
    }

    internal static void FreeNativeLibrary(IntPtr handle) => LibraryLoader.GetPlatformDefaultLoader().FreeNativeLibrary(handle);

    private static (string Platform, string Extension) GetResourcePlatform()
    {
        string platform;
        string extension;

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            platform = "win-";
            extension = ".dll";
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            platform = "osx-";
            extension = ".dylib";
        }
        else
        {
            platform = "linux-";
            extension = ".so";
        }

        if (RuntimeInformation.OSArchitecture == Architecture.X86)
        {
            platform += "x86";
        }
        else if (RuntimeInformation.OSArchitecture == Architecture.X64)
        {
            platform += "x64";
        }
        else if (RuntimeInformation.OSArchitecture == Architecture.Arm64)
        {
            platform += "arm64";
        }
        return (platform, extension);
    }

    private static string ExtractResourceToFile(string dllName)
    {
        var (platform, extension) = GetResourcePlatform();
        var resourceName = $"{platform}.{DllName}{extension}";

        // Get library bytes
        var assembly = Assembly.GetExecutingAssembly();
        using var resourceStream = assembly.GetManifestResourceStream(resourceName);
        if (resourceStream == null)
        {
            throw new Exception($"Resource '{resourceName}' not found in assembly.");
        }
        using var memoryStream = new MemoryStream();
        resourceStream.CopyTo(memoryStream);
        var bytes = memoryStream.ToArray();

        var hash = SHA1.Create().ComputeHash(bytes);
        var hashString = Convert.ToBase64String(hash)
            .Replace('+', '-') // replace URL unsafe characters with safe ones
            .Replace('/', '_') // replace URL unsafe characters with safe ones
            .Replace("=", "") // no padding
            .ToLowerInvariant();

        // Determine where to extract the DLL
        var tempDirectory = Path.Combine(Path.GetTempPath(), DllName, hashString);
        Directory.CreateDirectory(tempDirectory);

        var nativeLibraryPath = Path.Combine(tempDirectory, DllName + extension);
        // Extract the DLL only if it doesn't already exist
        if (!File.Exists(nativeLibraryPath))
        {
            using var fileStream = new FileStream(nativeLibraryPath, FileMode.Create, FileAccess.Write);
            fileStream.Write(bytes, 0, bytes.Length);
        }
        return nativeLibraryPath;
    }
}
