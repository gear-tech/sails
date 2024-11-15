#nullable disable
using System.Collections.Generic;
using System.IO;
using Microsoft.Build.Framework;
using Microsoft.Build.Utilities;
using Sails.ClientGenerator.Native;

namespace Sails.ClientBuildTask;

public class SailsIdl : Task
{
    //List of files
    [Required]
    public ITaskItem[] IdlFiles { get; set; }

    //The name of the namespace where the class is going to be generated
    [Required]
    public string IdlNamespace { get; set; }

    //The filename where the class was generated
    [Output]
    public string[] IdlGeneratedFiles { get; set; }

    public override bool Execute()
    {
        this.Log.LogMessage(MessageImportance.High, "Running Sails.NET IDL Code Generator");
        var list = new List<string>();

        foreach (var item in this.IdlFiles)
        {
            var filePath = item.GetMetadata("FullPath");

            this.Log.LogMessage(MessageImportance.High, "Reading \"" + filePath + "\".");
            var text = File.ReadAllText(filePath);
            var name = Path.GetFileNameWithoutExtension(filePath);

            var code = Generator.GenerateCode(text, new GeneratorConfig(name, this.IdlNamespace));

            var generatedName = $"{filePath}.generated.cs";
            File.Delete(generatedName);
            File.WriteAllText(generatedName, code);
            list.Add(generatedName);
            this.Log.LogMessage(MessageImportance.High, "Generated \"" + generatedName + "\".");
        }

        this.IdlGeneratedFiles = [.. list];
        return true;
    }
}
