namespace Il4ilSharp.Interop;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL error messages.</summary>
public unsafe static class Error
{
    public readonly ref struct Opaque { }

    [DllImport(Native.LibraryName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_error_dispose", ExactSpelling = true)]
    public static void Dispose(Opaque* message);
}
