import smartpi;
def main():
    livestream=smartpi.startup_livestream()
    camera=smartpi.startup_camera(640, 480, 50)
    inferer=smartpi.startup_inferer(livestream, "inference_graph.xml","inference_graph.bin")
    smartpi.startup_websocket(livestream, camera, inferer)
    while True:
        pass


main()
