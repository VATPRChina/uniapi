using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Configuration.Json;
using Tomlyn;
using Tomlyn.Model;

namespace Net.Vatprc.Uniapi.Utils.Toml;

// The following code is based on the work of
// https://github.dev/alexinea/Extensions.Configuration.Toml, and is originally
// licensed under the Apache License v2.0.

/// <summary>
/// A Toml file based <see cref="FileConfigurationProvider"/>.
/// </summary>
public class TomlConfigurationProvider : FileConfigurationProvider
{

    /// <summary>
    /// Constructor.
    /// </summary>
    /// <param name="source">The <see cref="TomlConfigurationSource"/>.</param>
    public TomlConfigurationProvider(TomlConfigurationSource source) : base(source) { }

    /// <summary>
    /// Loads Toml configuration key/values from a stream into a provider.
    /// </summary>
    /// <param name="stream">The toml <see cref="Stream"/> to load configuration data from.</param>
    public override void Load(Stream stream)
    {
        Data = TomlConfigurationFileParser.Parse(stream)!;
    }
}

/// <summary>
/// Represents a TOML file as an <see cref="IConfigurationSource"/>.
/// </summary>
public class TomlConfigurationSource : FileConfigurationSource
{
    /// <summary>
    /// Builds the <see cref="TomlConfigurationProvider"/> for this source.
    /// </summary>
    /// <param name="builder">The <see cref="IConfigurationBuilder"/>.</param>
    /// <returns>An <see cref="TomlConfigurationProvider"/></returns>
    public override IConfigurationProvider Build(IConfigurationBuilder builder)
    {
        EnsureDefaults(builder);
        return new TomlConfigurationProvider(this);
    }
}

internal class TomlConfigurationFileParser : Tomlyn.Syntax.SyntaxVisitor
{
    protected IDictionary<string, string> Data = new Dictionary<string, string>();

    public static IDictionary<string, string> Parse(Stream input)
        => new TomlConfigurationFileParser().ParseStream(input);

    private IDictionary<string, string> ParseStream(Stream stream)
    {
        Data.Clear();

        using var reader = new StreamReader(stream);
        var toml = Tomlyn.Toml.Parse(reader.ReadToEnd()).ToModel();
        // A TOML table maps to a TomlTable object and is in practice a IDictionary<string, object?>.
        // A TOML table array maps to a TomlTableArray object
        // A TOML array maps to a TomlArray object and is in practice a IList<object?>.
        // Floats/Double are all mapped to C# double.
        // Integers are all mapped to C# long.
        // Comments are preserved by default as TomlTable is implementing
        //   ITomlMetadataProvider (See Convert a runtime model to TOML string for more details).

        Convert("", toml);

        return Data;
    }

    private void Convert(string prefix, object? value)
    {
        switch (value)
        {
            case TomlTable v: Convert(prefix, v); break;
            case TomlTableArray v: Convert(prefix, v); break;
            case TomlArray v: Convert(prefix, v); break;
            case double v: Convert(prefix, v); break;
            case long v: Convert(prefix, v); break;
            case string v: Convert(prefix, v); break;
            case bool v: Convert(prefix, v); break;
            case TomlDateTime v: Convert(prefix, v); break;
            case null: break;
            default: throw new NotImplementedException($"{value.GetType().FullName} is not supported");
        }
    }

    private void Convert(string prefix, TomlTable value)
    {
        foreach (var (key, val) in value)
        {
            if (string.IsNullOrWhiteSpace(prefix)) Convert(key, val);
            else Convert($"{prefix}:{key}", val);
        }
    }

    private void Convert(string prefix, TomlTableArray value)
    {
        for (var i = 0; i < value.Count; i++)
        {
            if (string.IsNullOrWhiteSpace(prefix)) Convert(i.ToString(), value[i]);
            else Convert($"{prefix}:{i}", value[i]);
        }
    }

    private void Convert(string prefix, TomlArray value)
    {
        for (var i = 0; i < value.Count; i++)
        {
            if (string.IsNullOrWhiteSpace(prefix)) Convert(i.ToString(), value[i]);
            else Convert($"{prefix}:{i}", value[i]);
        }
    }

    private void Convert(string prefix, double value)
    {
        Data[prefix] = value.ToString(CultureInfo.InvariantCulture);
    }

    private void Convert(string prefix, long value)
    {
        Data[prefix] = value.ToString(CultureInfo.InvariantCulture);
    }

    private void Convert(string prefix, string value)
    {
        Data[prefix] = value;
    }

    private void Convert(string prefix, bool value)
    {
        Data[prefix] = value.ToString(CultureInfo.InvariantCulture);
    }

    private void Convert(string prefix, TomlDateTime value)
    {
        Data[prefix] = value.ToString();
    }
}

public static class TomlConfigurationExtensions
{
    public static IConfigurationBuilder AddTomlFile(
        this IConfigurationBuilder builder,
        string path)
    {
        return AddTomlFile(
            builder,
            provider: null,
            path: path,
            optional: false,
            reloadOnChange: false);
    }

    public static IConfigurationBuilder AddTomlFile(
        this IConfigurationBuilder builder,
        string path,
        bool optional)
    {
        return AddTomlFile(
            builder,
            provider: null,
            path: path,
            optional: optional,
            reloadOnChange: false);
    }

    public static IConfigurationBuilder AddTomlFile(
        this IConfigurationBuilder builder,
        Microsoft.Extensions.FileProviders.IFileProvider? provider,
        string path,
        bool optional,
        bool reloadOnChange)
    {
        if (builder is null) throw new ArgumentNullException(nameof(builder));
        if (string.IsNullOrEmpty(path)) throw new ArgumentException("Invalid file path.", nameof(path));

        return builder.AddTomlFile(s =>
        {
            s.FileProvider = provider;
            s.Path = path;
            s.Optional = optional;
            s.ReloadOnChange = reloadOnChange;
            s.ResolveFileProvider();
        });
    }

    public static IConfigurationBuilder AddTomlFile(
        this IConfigurationBuilder builder,
        Action<TomlConfigurationSource>? configureSource)
    {
        builder.Add(configureSource);
        return builder;
    }

    public static IConfigurationBuilder ReplaceJsonWithToml(this IConfigurationBuilder builder)
    {
        for (var i = 0; i < builder.Sources.Count; i++)
        {
            if (builder.Sources[i] is JsonConfigurationSource j && (j.Path?.StartsWith("appsettings") ?? false))
            {
                var s = new TomlConfigurationSource
                {
                    FileProvider = j.FileProvider,
                    OnLoadException = j.OnLoadException,
                    Optional = j.Optional,
                    ReloadDelay = j.ReloadDelay,
                    ReloadOnChange = j.ReloadOnChange,
                };
                if (j.Path != null) s.Path = Path.ChangeExtension(j.Path, "toml");
                s.ResolveFileProvider();
                builder.Sources[i] = s;
            }
        }
        return builder;
    }
}
