#nullable disable
using Microsoft.Build.Framework;
using Microsoft.Build.Utilities;

namespace Sails.ClientBuildTask;
public class SailsIdl : Task
{
    //List of files
    [Required]
    public ITaskItem[] IdlFiles { get; set; }

    //The name of the namespace where the class is going to be generated
    public string NamespaceName { get; set; }

    //The filename where the class was generated
    [Output]
    public string[] IdlGeneratedFiles { get; set; }

    public override bool Execute()
    {
        return true;
    }
}
