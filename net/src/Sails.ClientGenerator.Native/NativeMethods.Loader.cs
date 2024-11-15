#pragma warning disable RS1035 // Do not use APIs banned for analyzers

namespace Sails.ClientGenerator;

internal static unsafe partial class NativeMethods
{
    internal static NativeLibrary LoadNativeLibrary()
    {
        // Determine where to extract the DLL
        var tempDirectory = Path.Combine(Path.GetTempPath(), DllName);
        Directory.CreateDirectory(tempDirectory);

        var (platform, extension) = GetResourcePlatform();
        var nativeLibraryPath = Path.Combine(tempDirectory, DllName + extension);
        // Extract the DLL only if it doesn't already exist
        if (!File.Exists(nativeLibraryPath))
        {
            ExtractResourceToFile($"{platform}.{DllName}{extension}", nativeLibraryPath);
        }
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

    private static void ExtractResourceToFile(string resourceName, string filePath)
    {
        var assembly = Assembly.GetExecutingAssembly();
        using var resourceStream = assembly.GetManifestResourceStream(resourceName);
        if (resourceStream == null)
        {
            throw new Exception($"Resource '{resourceName}' not found in assembly.");
        }
        using var fileStream = new FileStream(filePath, FileMode.Create, FileAccess.Write);
        resourceStream.CopyTo(fileStream);
    }
}
