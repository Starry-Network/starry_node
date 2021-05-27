initSidebarItems({"constant":[["BLOCKSIZELOG_MAX",""],["BLOCKSIZE_MAX",""],["CHAINLOG_MAX_32",""],["CHAINLOG_MAX_64",""],["CHAINLOG_MIN",""],["CLEVEL_DEFAULT","Default compression level."],["CONTENTSIZE_ERROR",""],["CONTENTSIZE_UNKNOWN",""],["HASHLOG3_MAX",""],["HASHLOG_MIN",""],["LDM_BUCKETSIZELOG_MAX",""],["LDM_MINMATCH_MAX",""],["LDM_MINMATCH_MIN",""],["MAGICNUMBER",""],["MAGIC_DICTIONARY",""],["MAGIC_SKIPPABLE_START",""],["SEARCHLOG_MIN",""],["TARGETLENGTH_MAX",""],["TARGETLENGTH_MIN",""],["VERSION_MAJOR",""],["VERSION_MINOR",""],["VERSION_NUMBER",""],["VERSION_RELEASE",""],["WINDOWLOG_MAX_32",""],["WINDOWLOG_MAX_64",""],["WINDOWLOG_MIN",""]],"enum":[["CParameter","A compression parameter."],["DParameter","A decompression parameter."],["FrameFormat",""],["ResetDirective","Reset directive."],["Strategy","How to compress data."]],"fn":[["cctx_load_dictionary","Wraps the `ZSTD_CCtx_loadDictionary()` function."],["cctx_ref_cdict","Wraps the `ZSTD_CCtx_refCDict()` function."],["cctx_ref_prefix","Wraps the `ZSTD_CCtx_refPrefix()` function."],["cctx_reset","Wraps the `ZSTD_CCtx_reset()` function."],["cctx_set_parameter","Wraps the `ZSTD_CCtx_setParameter()` function."],["cctx_set_pledged_src_size","Wraps the `ZSTD_CCtx_setPledgedSrcSize()` function."],["compress","Wraps the `ZSTD_compress` function."],["compress2","Wraps the `ZSTD_compress2()` function."],["compress_block","Wraps the `ZSTD_compressBlock()` function."],["compress_bound","maximum compressed size in worst case single-pass scenario"],["compress_cctx","Wraps the `ZSTD_compressCCtx()` function"],["compress_stream","Wraps the `ZSTD_compressStream()` function."],["compress_stream2","Wraps the `ZSTD_compressStream2()` function."],["compress_using_cdict","Wraps the `ZSTD_compress_usingCDict()` function."],["compress_using_dict","Wraps the `ZSTD_compress_usingDict()` function."],["create_cctx",""],["create_cdict","Wraps the `ZSTD_createCDict()` function."],["create_cdict_by_reference","Wraps the `ZSTD_createCDict_byReference()` function."],["create_cstream","Allocates a new `CStream`."],["create_dctx","Prepares a new decompression context without dictionary."],["create_ddict","Wraps the `ZSTD_createDDict()` function."],["create_ddict_by_reference","Wraps the `ZSTD_createDDict_byReference()` function."],["create_dstream",""],["cstream_in_size","Wraps `ZSTD_CStreamInSize()`"],["cstream_out_size","Wraps `ZSTD_CStreamOutSize()`"],["dctx_load_dictionary","Wraps the `ZSTD_DCtx_loadDictionary()` function."],["dctx_ref_ddict","Wraps the `ZSTD_DCtx_refDDict()` function."],["dctx_ref_prefix","Wraps the `ZSTD_DCtx_refPrefix()` function."],["dctx_reset","Wraps the `ZSTD_DCtx_reset()` function."],["dctx_set_parameter","Wraps the `ZSTD_DCtx_setParameter()` function."],["decompress","Wraps the `ZSTD_decompress` function."],["decompress_block","Wraps the `ZSTD_decompressBlock()` function."],["decompress_dctx","Wraps the `ZSTD_decompressDCtx()` function."],["decompress_stream","Wraps the `ZSTD_decompressStream()` function."],["decompress_using_ddict","Wraps the `ZSTD_decompress_usingDDict()` function."],["decompress_using_dict","Wraps the `ZSTD_decompress_usingDict()` function."],["dstream_in_size","Wraps the `ZSTD_DStreamInSize()` function."],["dstream_out_size","Wraps the `ZSTD_DStreamOutSize()` function."],["end_stream","Wraps the `ZSTD_endStream()` function."],["find_decompressed_size","Wraps the `ZSTD_findDecompressedSize()` function."],["find_frame_compressed_size","Wraps the `ZSTD_findFrameCompressedSize()` function."],["flush_stream","Wraps the `ZSTD_flushStream()` function."],["get_block_size","Wraps the `ZSTD_getBlockSize()` function."],["get_decompressed_size","Wraps the `ZSTD_getDecompressedSize` function."],["get_dict_id","Wraps the `ZSTD_getDictID_fromDict()` function."],["get_dict_id_from_ddict","Wraps the `ZSTD_getDictID_fromDDict()` function."],["get_dict_id_from_dict","Wraps the `ZSTD_getDictID_fromDict()` function."],["get_dict_id_from_frame","Wraps the `ZSTD_getDictID_fromFrame()` function."],["get_error_name",""],["get_frame_content_size","Wraps the `ZSTD_getFrameContentSize()` function."],["init_cstream","Prepares an existing `CStream` for compression at the given level."],["init_cstream_src_size","Wraps the `ZSTD_initCStream_srcSize()` function."],["init_cstream_using_cdict","Wraps the `ZSTD_initCStream_usingCDict()` function."],["init_cstream_using_dict","Wraps the `ZSTD_initCStream_usingDict()` function."],["init_dstream","Wraps the `ZSTD_initCStream()` function."],["init_dstream_using_ddict","Wraps the `ZSTD_initDStream_usingDDict()` function."],["init_dstream_using_dict","Wraps the `ZSTD_initDStream_usingDict()` function."],["insert_block","Wraps the `ZSTD_insertBlock()` function."],["is_frame","Wraps the `ZSTD_isFrame()` function."],["max_c_level","Returns the maximum (slowest) compression level supported."],["min_c_level","Returns the minimum (fastest) compression level supported."],["reset_cstream","Wraps the `ZSTD_resetCStream()` function."],["reset_dstream","Wraps the `ZSTD_resetDStream()` function."],["sizeof_cctx","Wraps the `ZSTD_sizeofCCtx()` function."],["sizeof_cdict","Wraps the `ZSTD_sizeof_CDict()` function."],["sizeof_cstream","Wraps the `ZSTD_sizeof_CStream()` function."],["sizeof_dctx","Wraps the `ZSTD_sizeof_DCtx()` function."],["sizeof_ddict","Wraps the `ZSTD_sizeof_DDict()` function."],["sizeof_dstream","Wraps the `ZSTD_sizeof_DStream()` function."],["train_from_buffer","Wraps thge `ZDICT_trainFromBuffer()` function."],["version_number",""],["version_string",""]],"struct":[["CCtx",""],["CDict","Compression dictionary."],["DCtx","A Decompression Context."],["DDict",""],["InBuffer","Wrapper around an input buffer."],["OutBuffer","Wrapper around an output buffer."]],"type":[["CStream","Compression stream."],["CompressionLevel","Represents the compression level used by zstd."],["DStream","A Decompression stream."],["ErrorCode","Represents a possible error from the zstd library."],["SafeResult","Wrapper result around most zstd functions."]]});