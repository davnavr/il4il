namespace Il4ilSharp.Interop;

using Il4ilSharp.Interop.Native;

internal unsafe static class ErrorHandling {
    internal static void Throw(Error.Opaque* error) {
        if (error != null) {
            try {
                throw new Il4ilSharp.ErrorHandlingException("TODO: Get error message contents");
            } finally {
                Error.Dispose(error);
            }
        }
    }
}
