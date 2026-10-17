# Builds ../../rust with Cargokit before compiling, then force-loads the
# static library into this pod so the Rust symbols are available to dart:ffi.
Pod::Spec.new do |s|
  s.name             = 'jixel_native'
  s.version          = '0.1.0'
  s.summary          = 'The Jixel Rust engine, built with Cargokit.'
  s.homepage         = 'https://github.com/lofiorin/jixel'
  s.license          = { :type => 'Apache-2.0' }
  s.author           = { 'Jixel contributors' => 'https://github.com/lofiorin/jixel' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'FlutterMacOS'
  s.platform = :osx, '10.15'
  s.swift_version = '5.0'

  s.script_phase = {
    :name => 'Build Rust library',
    # First argument: path to the Rust crate relative to this podspec.
    :script => 'sh "$PODS_TARGET_SRCROOT/../cargokit/build_pod.sh" ../../rust jixel_bridge',
    :execution_position => :before_compile,
    :input_files => ['${BUILT_PRODUCTS_DIR}/cargokit_phony'],
    # Let Xcode know the static lib is an output so it waits for it.
    :output_files => ["${BUILT_PRODUCTS_DIR}/libjixel_bridge.a"],
  }
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',

    # libjxl is C++: link the C++ runtime alongside the Rust library.
    'OTHER_LDFLAGS' => '-force_load ${BUILT_PRODUCTS_DIR}/libjixel_bridge.a -lc++',
  }
end
