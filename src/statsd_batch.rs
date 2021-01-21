#[allow(dead_code)]
pub mod statsd_batch {
    #[derive(Copy, Clone)]
    pub struct Owned;
    impl <'a> ::capnp::traits::Owned<'a> for Owned { type Reader = Reader<'a>; type Builder = Builder<'a>; }
    impl <'a> ::capnp::traits::OwnedStruct<'a> for Owned { type Reader = Reader<'a>; type Builder = Builder<'a>; }
    impl ::capnp::traits::Pipelined for Owned { type Pipeline = Pipeline; }

    #[derive(Clone, Copy)]
    pub struct Reader<'a> { reader: ::capnp::private::layout::StructReader<'a> }

    impl <'a,> ::capnp::traits::HasTypeId for Reader<'a,>  {
        #[inline]
        fn type_id() -> u64 { _private::TYPE_ID }
    }
    impl <'a,> ::capnp::traits::FromStructReader<'a> for Reader<'a,>  {
        fn new(reader: ::capnp::private::layout::StructReader<'a>) -> Reader<'a,> {
            Reader { reader: reader,  }
        }
    }

    impl <'a,> ::capnp::traits::FromPointerReader<'a> for Reader<'a,>  {
        fn get_from_pointer(reader: &::capnp::private::layout::PointerReader<'a>) -> ::capnp::Result<Reader<'a,>> {
            ::std::result::Result::Ok(::capnp::traits::FromStructReader::new(reader.get_struct(::std::ptr::null())?))
        }
    }

    impl <'a,> ::capnp::traits::IntoInternalStructReader<'a> for Reader<'a,>  {
        fn into_internal_struct_reader(self) -> ::capnp::private::layout::StructReader<'a> {
            self.reader
        }
    }

    impl <'a,> ::capnp::traits::Imbue<'a> for Reader<'a,>  {
        fn imbue(&mut self, cap_table: &'a ::capnp::private::layout::CapTable) {
            self.reader.imbue(::capnp::private::layout::CapTableReader::Plain(cap_table))
        }
    }

    impl <'a,> Reader<'a,>  {
        pub fn reborrow(&self) -> Reader<> {
            Reader { .. *self }
        }

        pub fn total_size(&self) -> ::capnp::Result<::capnp::MessageSize> {
            self.reader.total_size()
        }
        #[inline]
        pub fn get_metric_labels(self) -> ::capnp::Result<::capnp::text_list::Reader<'a>> {
            ::capnp::traits::FromPointerReaderRefDefault::get_from_pointer(&self.reader.get_pointer_field(0), ::std::ptr::null())
        }
        pub fn has_metric_labels(&self) -> bool {
            !self.reader.get_pointer_field(0).is_null()
        }
        #[inline]
        pub fn get_metric_values(self) -> ::capnp::Result<::capnp::primitive_list::Reader<'a,f64>> {
            ::capnp::traits::FromPointerReaderRefDefault::get_from_pointer(&self.reader.get_pointer_field(1), ::std::ptr::null())
        }
        pub fn has_metric_values(&self) -> bool {
            !self.reader.get_pointer_field(1).is_null()
        }
        #[inline]
        pub fn get_metric_kinds(self) -> ::capnp::Result<::capnp::enum_list::Reader<'a,MetricKind>> {
            ::capnp::traits::FromPointerReaderRefDefault::get_from_pointer(&self.reader.get_pointer_field(2), ::std::ptr::null())
        }
        pub fn has_metric_kinds(&self) -> bool {
            !self.reader.get_pointer_field(2).is_null()
        }
    }

    pub struct Builder<'a> { builder: ::capnp::private::layout::StructBuilder<'a> }
    impl <'a,> ::capnp::traits::HasStructSize for Builder<'a,>  {
        #[inline]
        fn struct_size() -> ::capnp::private::layout::StructSize { _private::STRUCT_SIZE }
    }
    impl <'a,> ::capnp::traits::HasTypeId for Builder<'a,>  {
        #[inline]
        fn type_id() -> u64 { _private::TYPE_ID }
    }
    impl <'a,> ::capnp::traits::FromStructBuilder<'a> for Builder<'a,>  {
        fn new(builder: ::capnp::private::layout::StructBuilder<'a>) -> Builder<'a, > {
            Builder { builder: builder,  }
        }
    }

    impl <'a,> ::capnp::traits::ImbueMut<'a> for Builder<'a,>  {
        fn imbue_mut(&mut self, cap_table: &'a mut ::capnp::private::layout::CapTable) {
            self.builder.imbue(::capnp::private::layout::CapTableBuilder::Plain(cap_table))
        }
    }

    impl <'a,> ::capnp::traits::FromPointerBuilder<'a> for Builder<'a,>  {
        fn init_pointer(builder: ::capnp::private::layout::PointerBuilder<'a>, _size: u32) -> Builder<'a,> {
            ::capnp::traits::FromStructBuilder::new(builder.init_struct(_private::STRUCT_SIZE))
        }
        fn get_from_pointer(builder: ::capnp::private::layout::PointerBuilder<'a>) -> ::capnp::Result<Builder<'a,>> {
            ::std::result::Result::Ok(::capnp::traits::FromStructBuilder::new(builder.get_struct(_private::STRUCT_SIZE, ::std::ptr::null())?))
        }
    }

    impl <'a,> ::capnp::traits::SetPointerBuilder<Builder<'a,>> for Reader<'a,>  {
        fn set_pointer_builder<'b>(pointer: ::capnp::private::layout::PointerBuilder<'b>, value: Reader<'a,>, canonicalize: bool) -> ::capnp::Result<()> { pointer.set_struct(&value.reader, canonicalize) }
    }

    impl <'a,> Builder<'a,>  {
        #[deprecated(since="0.9.2", note="use into_reader()")]
        pub fn as_reader(self) -> Reader<'a,> {
            self.into_reader()
        }
        pub fn into_reader(self) -> Reader<'a,> {
            ::capnp::traits::FromStructReader::new(self.builder.into_reader())
        }
        pub fn reborrow(&mut self) -> Builder<> {
            Builder { .. *self }
        }
        pub fn reborrow_as_reader(&self) -> Reader<> {
            ::capnp::traits::FromStructReader::new(self.builder.into_reader())
        }

        pub fn total_size(&self) -> ::capnp::Result<::capnp::MessageSize> {
            self.builder.into_reader().total_size()
        }
        #[inline]
        pub fn get_metric_labels(self) -> ::capnp::Result<::capnp::text_list::Builder<'a>> {
            ::capnp::traits::FromPointerBuilderRefDefault::get_from_pointer(self.builder.get_pointer_field(0), ::std::ptr::null())
        }
        #[inline]
        pub fn set_metric_labels(&mut self, value: ::capnp::text_list::Reader<'a>) -> ::capnp::Result<()> {
            ::capnp::traits::SetPointerBuilder::set_pointer_builder(self.builder.get_pointer_field(0), value, false)
        }
        #[inline]
        pub fn init_metric_labels(self, size: u32) -> ::capnp::text_list::Builder<'a> {
            ::capnp::traits::FromPointerBuilder::init_pointer(self.builder.get_pointer_field(0), size)
        }
        pub fn has_metric_labels(&self) -> bool {
            !self.builder.get_pointer_field(0).is_null()
        }
        #[inline]
        pub fn get_metric_values(self) -> ::capnp::Result<::capnp::primitive_list::Builder<'a,f64>> {
            ::capnp::traits::FromPointerBuilderRefDefault::get_from_pointer(self.builder.get_pointer_field(1), ::std::ptr::null())
        }
        #[inline]
        pub fn set_metric_values(&mut self, value: ::capnp::primitive_list::Reader<'a,f64>) -> ::capnp::Result<()> {
            ::capnp::traits::SetPointerBuilder::set_pointer_builder(self.builder.get_pointer_field(1), value, false)
        }
        #[inline]
        pub fn init_metric_values(self, size: u32) -> ::capnp::primitive_list::Builder<'a,f64> {
            ::capnp::traits::FromPointerBuilder::init_pointer(self.builder.get_pointer_field(1), size)
        }
        pub fn has_metric_values(&self) -> bool {
            !self.builder.get_pointer_field(1).is_null()
        }
        #[inline]
        pub fn get_metric_kinds(self) -> ::capnp::Result<::capnp::enum_list::Builder<'a,MetricKind>> {
            ::capnp::traits::FromPointerBuilderRefDefault::get_from_pointer(self.builder.get_pointer_field(2), ::std::ptr::null())
        }
        #[inline]
        pub fn set_metric_kinds(&mut self, value: ::capnp::enum_list::Reader<'a,MetricKind>) -> ::capnp::Result<()> {
            ::capnp::traits::SetPointerBuilder::set_pointer_builder(self.builder.get_pointer_field(2), value, false)
        }
        #[inline]
        pub fn init_metric_kinds(self, size: u32) -> ::capnp::enum_list::Builder<'a,MetricKind> {
            ::capnp::traits::FromPointerBuilder::init_pointer(self.builder.get_pointer_field(2), size)
        }
        pub fn has_metric_kinds(&self) -> bool {
            !self.builder.get_pointer_field(2).is_null()
        }
    }

    pub struct Pipeline { _typeless: ::capnp::any_pointer::Pipeline }
    impl ::capnp::capability::FromTypelessPipeline for Pipeline {
        fn new(typeless: ::capnp::any_pointer::Pipeline) -> Pipeline {
            Pipeline { _typeless: typeless,  }
        }
    }
    impl Pipeline  {
    }
    mod _private {
        use capnp::private::layout;
        pub const STRUCT_SIZE: layout::StructSize = layout::StructSize { data: 0, pointers: 3 };
        pub const TYPE_ID: u64 = 0xe5b5_1f8a_bb7a_7979;
    }

    #[repr(u16)]
    #[derive(Clone, Copy, PartialEq)]
    pub enum MetricKind {
        Gauge = 0,
        Counter = 1,
        Timer = 2,
    }
    impl ::capnp::traits::FromU16 for MetricKind {
        #[inline]
        fn from_u16(value: u16) -> ::std::result::Result<MetricKind, ::capnp::NotInSchema> {
            match value {
                0 => ::std::result::Result::Ok(MetricKind::Gauge),
                1 => ::std::result::Result::Ok(MetricKind::Counter),
                2 => ::std::result::Result::Ok(MetricKind::Timer),
                n => ::std::result::Result::Err(::capnp::NotInSchema(n)),
            }
        }
    }
    impl ::capnp::traits::ToU16 for MetricKind {
        #[inline]
        fn to_u16(self) -> u16 { self as u16 }
    }
    impl ::capnp::traits::HasTypeId for MetricKind {
        #[inline]
        fn type_id() -> u64 { 0xfa6b_abd2_3649_9d98u64 }
    }
}