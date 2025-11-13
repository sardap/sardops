import time
from datetime import datetime, timedelta
import pygame
from sdop_py import GamePy, GameTime
from io import BytesIO


def now_game_time() -> GameTime:
    now = datetime.now()
    return GameTime(now.year, now.month, now.day, now.hour, now.minute, now.second)


def main():
    scale = 12
    width = 64 * scale
    height = 128 * scale

    pygame.init()
    game = GamePy(now_game_time())
    screen = pygame.display.set_mode((width, height))

    sdop_filename = "sdop.sav"

    try:
        with open(sdop_filename, "rb") as f:
            data = f.read()
        game = GamePy.load_from_save(now_game_time(), data)
    except FileNotFoundError:
        print("File does not exist.")

    left_btn, middle_btn, right_btn = (False, False, False)
    last_save = datetime.now()
    running = True
    last = time.time_ns()
    while running:
        now = time.time_ns()
        delta = now - last
        last = now

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_q:
                    left_btn = True
                elif event.key == pygame.K_w:
                    middle_btn = True
                elif event.key == pygame.K_e:
                    right_btn = True
                elif event.key == pygame.K_ESCAPE:
                    running = False
            elif event.type == pygame.KEYUP:
                if event.key == pygame.K_q:
                    left_btn = False
                elif event.key == pygame.K_w:
                    middle_btn = False
                elif event.key == pygame.K_e:
                    right_btn = False

        game.update_inputs(left_btn, middle_btn, right_btn)
        game.tick(delta)
        game.refresh_display(delta)

        save_delta = datetime.now() - last_save
        if save_delta > timedelta(minutes=1):
            last_save = datetime.now()
            save_bytes = game.get_save_bytes(now_game_time())
            if save_bytes:
                with open(sdop_filename, "wb") as f:
                    f.write(save_bytes)

        bmp = game.display_bitmap()
        img = pygame.image.load(BytesIO(bmp)).convert()
        img = pygame.transform.scale(img, (width, height))
        screen.blit(img, (0, 0))
        pygame.display.flip()

    pygame.quit()


if __name__ == "__main__":
    main()
