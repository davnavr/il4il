namespace Il4ilSharp;

using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Il4ilSharp.Interop;

/// <summary>Represents an IL4IL module which has been validated.</summary>
/// <remarks>This class provides various methods to examine the contents of IL4IL modules.</remarks>
public sealed class ValidModule {
    /// <summary>Gets the underlying handle to the browser used to read the content of the IL4IL module.</summary>
    public BrowserHandle Browser { get; } // TODO: Maybe have an interface for this so inheritdoc can be used?

    /// <summary>Gets the contents of the module's metadata section.</summary>
    public IReadOnlyList<ModuleMetadata> Metadata { get; }

    private static IReadOnlyList<ModuleMetadata> InitializeMetadata(BrowserHandle browser) {
        var handles = browser.GetMetadata();
        var metadata = new ModuleMetadata[handles.Length];
        for (int i = 0; i < metadata.Length; i++) {
            metadata[i] = ModuleMetadata.Create(handles[i]);
        }

        return new ReadOnlyCollection<ModuleMetadata>(metadata);
    }

    /// <summary>Initializes a <see cref="ValidModule"/> with the specified <see cref="BrowserHandle"/>.</summary>
    /// <exception cref="ArgumentNullException">Thrown if the <paramref name="browser"/> is <see langword="null"/>.</exception>
    public ValidModule(BrowserHandle browser) {
        ArgumentNullException.ThrowIfNull(nameof(browser));
        Browser = browser;
        Metadata = InitializeMetadata(browser);
    }

    /// <summary>Initializes a <see cref="ValidModule"/> from the contents of a <see cref="Interop.ModuleHandle"/>.</summary>
    public ValidModule(Interop.ModuleHandle module) : this(module.ValidateAndDispose()) { }
}
