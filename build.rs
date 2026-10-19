fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // This manifest tells Windows to use version 6 of comctl32.dll,
        // which provides modern visual styles and prevents the 
        // STATUS_ENTRYPOINT_NOT_FOUND crash in native-windows-gui.
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<dependency>
    <dependentAssembly>
        <assemblyIdentity
            type="win32"
            name="Microsoft.Windows.Common-Controls"
            version="6.0.0.0"
            processorArchitecture="*"
            publicKeyToken="6595b64144ccf1df"
            language="*"
        />
    </dependentAssembly>
</dependency>
</assembly>
"#);
        res.compile().unwrap();
    }
}
