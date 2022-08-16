namespace Il4ilSharp.Interop;

using System;

/// <summary>Provides a simple container for <see cref="Exception"/> instances.</summary>
public sealed class CaughtException {
    private Exception? thrown;

    /// <summary>Initializes a <see cref="CaughtException"/> instance without an exception that was caught.</summary>
    public CaughtException() {
        thrown = null;
    }

    /// <summary>Gets the <see cref="Exception"/> that was caught.</summary>
    public Exception? Thrown => thrown;

    /// <summary>Updates the current exception that was caught.</summary>
    /// <remarks>If an exception that was previously caught was not yet thrown, an <see cref="AggregateException"/> is used.</remarks>
    public void Catch(Exception e) {
        if (e == null) {
            return;
        }

        thrown = thrown == null ? e : new AggregateException(new Exception[] { e, thrown }).Flatten();
    }

    /// <summary>Throws the exception, if it was not <see langword="null"/>.</summary>
    public void ThrowAny() {
        Exception? e = thrown;
        if (e != null) {
            thrown = null;
            throw e;
        }
    }
}
