

impl fmt::Debug for Span {
    #[cfg(not(feature = "no-span-debug"))]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Span")
            .field("src (ptr)", &self.src.as_ptr())
            .field("source_id", &self.source_id)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("as_str()", &self.as_str())
            .finish()
    }
    #[cfg(feature = "no-span-debug")]
    fn fmt(&self, _fmt: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
