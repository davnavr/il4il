namespace Il4ilSharp;

using System;

/// <summary>The exception that is thrown when an error occurs in <c>il4il_c</c>.</summary>
public class ErrorHandlingException : Exception {
    /// <summary>Initializes an <see cref="ErrorHandlingException"/> instance with the specified <paramref name="message"/>.</summary>
    public ErrorHandlingException(string message) : base(message) { }
}
