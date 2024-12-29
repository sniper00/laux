---@diagnostic disable: undefined-global

workspace "extensions"
    configurations { "Debug", "Release" }
    flags{"NoPCH","RelativeLinks"}
    cppdialect "C++20"
    location "./"
    architecture "x64"
    -- staticruntime "on"

    filter "configurations:Debug"
        defines { "DEBUG" }
        symbols "On"

    filter "configurations:Release"
        defines { "NDEBUG" }
        optimize "On"
        symbols "On"

    filter {"system:windows"}
        characterset "MBCS"
        systemversion "latest"
        warnings "Extra"
        cdialect "C11"
        buildoptions{"/experimental:c11atomics"}

    filter { "system:linux" }
        warnings "High"

    filter { "system:macosx" }
        warnings "High"

--- Adds a Lua module to the build configuration.
---
--- This function configures a Lua module located in the specified directory
--- and assigns it the given name. Additional platform-specific options can
--- be provided to customize the build process for different operating systems.
---
--- @param dir string The path to the module's source files, relative to the current directory.
--- @param name string The name of the Lua module.
--- @param options? table Additional options for the module configuration.
---                      The options table can contain the following keys:
---                      - all: function() end - A function to be executed for all platforms.
---                      - windows: function() end - A function to be executed only on Windows.
---                      - linux: function() end - A function to be executed only on Linux.
---                      - macosx: function() end - A function to be executed only on macOS.
local function add_lua_module(dir, name, options )
    project(name)
        location("build/projects/%{prj.name}")
        objdir "build/obj/%{prj.name}/%{cfg.buildcfg}"
        targetdir "build/bin/%{cfg.buildcfg}"

        language "C"
        kind "SharedLib"
        includedirs {"./cpp", "moon/third/lua/"}
        libdirs {"moon/build/bin/Release"}
        files { dir.."/*.h",dir.."/*.hpp", dir.."/*.c",dir.."/*.cpp"}

        defines{"SOL_ALL_SAFETIES_ON"}

        if not options then
            options = {}
        end

        if type(options.all)=="function" then
            options.all()
        end

        filter { "system:windows" }
            defines{"LUA_BUILD_AS_DLL"}
            links{"moon"}
            if type(options.windows)=="function" then
                options.windows()
            end
        filter {"system:linux"}
            targetprefix ""
            if type(options.linux)=="function" then
                options.linux()
            end
        filter {"system:macosx"}
            targetprefix ""
            if type(options.macosx)=="function" then
                options.macosx()
            end
end

----------------------Lua C/C++ Modules------------------------

add_lua_module("./cpp/sproto", "sproto")
add_lua_module("./cpp/lpeg", "lpeg")
add_lua_module(
"3rd/math3d",
"math3d",
 {
    all = function ()
        includedirs("3rd/math3d/glm")
        language "C++"
        defines{"GLM_ENABLE_EXPERIMENTAL","GLM_FORCE_QUAT_DATA_XYZW","_USE_MATH_DEFINES"}
    end}
)

local function string_trim(input, chars)
    chars = chars or " \t\n\r"
    local pattern = "^[" .. chars .. "]+"
    input = string.gsub(input, pattern, "")
    pattern = "[" .. chars .. "]+$"
    return string.gsub(input, pattern, "")
end

newaction {
    trigger = "build",
    description = "Build",
    execute = function ()
        os.execute("git pull")
        os.execute("git submodule init")
        os.execute("git submodule update")
        --- Build moon
        os.execute([[cd moon && premake5 build]])

        -- Build Cpp library
        local host = os.host()
        local switch = {
            windows = function ()
                os.execute("premake5.exe vs2022")
                local command = os.getenv("ProgramFiles(x86)")..[[\Microsoft Visual Studio\Installer\vswhere.exe]]
                command = string.format('"%s" %s', string_trim(command), " -latest -products * -requires Microsoft.Component.MSBuild -property installationPath")
                local handle = assert(io.popen(command))
                command = handle:read("*a")
                handle:close()
                os.execute(string.format('"%s%s" -maxcpucount:4 extensions.sln /t:build /p:Configuration=Release ', string_trim(command), [[\MSBuild\Current\Bin\MSBuild.exe]]))
            end,
            linux = function ()
                os.execute("premake5 gmake2")
                os.execute( "make -j4 config=release")
            end,
            macosx = function ()
                os.execute("premake5 gmake2 --cc=clang")
                os.execute("make -j4 config=release")
            end,
        }

        switch[host]()

        -- Build Rust library
        os.execute([[cd rust && cargo build --release]])
    end
}

newaction {
    trigger = "publish",
    description = "Publish",
    execute = function ()
        os.mkdir("moon/clib")

        local host = os.host()
        local switch = {
            windows = function ()
                os.execute([[xcopy .\build\bin\Release\*.dll .\moon\clib\ /Y /F]])
                os.execute([[xcopy .\rust\target\release\*.dll .\moon\clib\ /Y /F]])
                os.execute([[xcopy .\lualib moon\lualib\ /E /Y /F]])
            end,
            linux = function ()
                os.execute([[cp -rf lualib moon]])
                os.execute([[cp -f ./build/bin/Release/*.so moon/clib/]])
                os.execute([[cp -f ./rust/target/release/librust.so moon/clib/rust.so]])
            end,
            macosx = function ()
                os.execute([[cp -rf lualib moon]])
                os.execute([[cp -f ./build/bin/Release/*.dylib moon/clib/]])
                os.execute([[cp -f ./rust/target/release/librust.dylib moon/clib/rust.dylib]])
            end,
        }

        switch[host]()

        os.execute([[cd moon && premake5 publish]])

        if os.host() == "windows" then
            os.execute('move moon\\*.zip .\\')
        else
            os.execute([[mv moon/*.zip ./]])
        end

    end
}