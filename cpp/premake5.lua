---@diagnostic disable: undefined-global

workspace "luax"
    configurations { "Debug", "Release" }
    flags{"NoPCH","RelativeLinks"}
    cppdialect "C++20"
    location "./"
    architecture "x64"
    staticruntime "on"

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

--[[
    lua C/C++模块
    @dir： 模块源文件所在路径，相对于当前目录的路径
    @name: LUAMOD name
    @normaladdon : 平台通用的附加项
    @winddowsaddon : windows下的附加项
    @linuxaddon : linux下的附加项
    @macaddon : macosx下的附加项

    使用：
    模块编写规范：使用 LUAMOD_API 导出符号(windows)

    注意：
    默认使用C编译器编译，可以使用 *addon 参数进行更改
]]
local function add_lua_module(dir, name, normaladdon, windowsaddon, linuxaddon, macaddon )
    project(name)
        location("build/projects/%{prj.name}")
        objdir "build/obj/%{prj.name}/%{cfg.buildcfg}"
        targetdir "build/bin/%{cfg.buildcfg}"
        language "C"
        kind "SharedLib"
        includedirs {"./", "./","../3rd/moon/third/lua/"}
        files { dir.."/*.h",dir.."/*.hpp", dir.."/*.c",dir.."/*.cpp"}

        defines{"SOL_ALL_SAFETIES_ON"}

        if type(normaladdon)=="function" then
            normaladdon()
        end
        libdirs {"../3rd/moon/build/bin/Release"}
        links{"lua"}
        filter { "system:windows" }
            defines{"LUA_BUILD_AS_DLL"}
            if type(windowsaddon)=="function" then
                windowsaddon()
            end
        filter {"system:linux"}
            targetprefix ""
            if type(linuxaddon)=="function" then
                linuxaddon()
            end
        filter {"system:macosx"}
            targetprefix ""
            if type(macaddon)=="function" then
                macaddon()
            end
end

----------------------Lua C/C++ Modules------------------------

add_lua_module("./sproto", "sproto")
add_lua_module("./lpeg", "lpeg")
add_lua_module("../3rd/math3d", "math3d", function ()
    includedirs("../3rd/math3d/glm")
    language "C++"
    defines{"GLM_ENABLE_EXPERIMENTAL","GLM_FORCE_QUAT_DATA_XYZW","_USE_MATH_DEFINES"}
end)
