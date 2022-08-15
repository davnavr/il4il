namespace Il4ilSharp;

using System;
using Il4ilSharp.Interop;

/// <summary>
/// Represents an IL4IL module which has been validated.
/// </summary>
/// <remarks>This class provides various methods to examine the contents of IL4IL modules.</remarks>
public sealed class ValidModule {
    /// <summary>Gets the underlying handle to the browser used to read the content of the IL4IL module.</summary>
    public BrowserHandle Browser { get; } // TODO: Maybe have an interface for this so inheritdoc can be used?

    internal ValidModule(BrowserHandle browser) {
        ArgumentNullException.ThrowIfNull(nameof(browser));

        try {
            unsafe {
                browser.Enter(); // Prevent handle from being disposed early
            }

            Browser = browser;
        } finally {
            browser.Exit();
        }
    }

    public ValidModule(Interop.ModuleHandle module) : this(module.ValidateAndDispose()) { }
}
