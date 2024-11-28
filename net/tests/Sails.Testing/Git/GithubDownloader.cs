using System;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using EnsureThat;

namespace Sails.Testing.Git;

public sealed class GithubDownloader
{
    public GithubDownloader(string organization, string repository)
    {
        EnsureArg.IsNotNullOrWhiteSpace(organization, nameof(organization));
        EnsureArg.IsNotNullOrWhiteSpace(repository, nameof(repository));

        this.organization = organization;
        this.repository = repository;
    }

    private static readonly HttpClient HttpClient = new();

    private readonly string organization;
    private readonly string repository;

    public Task<Stream> DownloadFileAsync(string fileName, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNullOrWhiteSpace(fileName, nameof(fileName));

        return this.DownloadFileFromBranchAsync("master", fileName, cancellationToken);
    }

    public Task<Stream> DownloadFileFromTagAsync(string tag, string fileName, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNullOrWhiteSpace(fileName, nameof(fileName));
        EnsureArg.IsNotNullOrWhiteSpace(tag, nameof(tag));

        return this.DownloadFileAsync(refName: $"tags/{tag}", fileName, cancellationToken);
    }

    public Task<Stream> DownloadFileFromBranchAsync(string branch, string fileName, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNullOrWhiteSpace(fileName, nameof(fileName));
        EnsureArg.IsNotNullOrWhiteSpace(branch, nameof(branch));

        return this.DownloadFileAsync(refName: $"heads/{branch}", fileName, cancellationToken);
    }

    public Task<Stream> DownloadReleaseAssetAsync(string releaseTag, string assetName, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNullOrWhiteSpace(releaseTag, nameof(releaseTag));
        EnsureArg.IsNotNullOrWhiteSpace(assetName, nameof(assetName));

        var downloadUrl = this.BuildReleaseAssetDownloadUrl(releaseTag, assetName);
        return HttpClient.GetStreamAsync(downloadUrl, cancellationToken);
    }

    private Task<Stream> DownloadFileAsync(string refName, string fileName, CancellationToken cancellationToken)
    {
        EnsureArg.IsNotNullOrWhiteSpace(refName, nameof(refName));
        EnsureArg.IsNotNullOrWhiteSpace(fileName, nameof(fileName));

        var downloadUrl = this.BuildFileDownloadUrl(refName, fileName);
        return HttpClient.GetStreamAsync(downloadUrl, cancellationToken);
    }

    private Uri BuildFileDownloadUrl(string refName, string fileName)
    {
        var encodedRefName = WebUtility.UrlEncode(refName);
        var encodedFileName = WebUtility.UrlEncode(fileName);
        return new($"https://raw.githubusercontent.com/{this.organization}/{this.repository}/"
            + $"refs/{encodedRefName}/{encodedFileName}");
    }

    private Uri BuildReleaseAssetDownloadUrl(string releaseTag, string assetName)
    {
        var encodedReleaseTag = WebUtility.UrlEncode(releaseTag);
        var encodedAssetName = WebUtility.UrlEncode(assetName);
        return new($"https://github.com/{this.organization}/{this.repository}/"
            + $"releases/download/{encodedReleaseTag}/{encodedAssetName}");
    }
}
