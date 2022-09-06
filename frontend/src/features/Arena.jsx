import React from "react";
import { useEffect } from "react";
import { useSelector } from "react-redux";

import "../css/Arena.css";
import Gateway from "./Gateway";

export function Arena() {
	const arena_width = useSelector((state) => state.gameState.arena_width);
	const arena_height = useSelector((state) => state.gameState.arena_height);
	const players = useSelector((state) => state.gameState.players);
	const food = useSelector((state) => state.gameState.food);

	const gateway = new Gateway();

	useEffect(() => {
		document.getElementById("arena").focus();
	});

	const handleKeyDown = (e) => {
		let key = e.key;
		console.debug("You pressed a key: " + key);
		let direction = null;
		switch (key) {
			case "ArrowDown":
				direction = "Down";
				break;
			case "ArrowUp":
				direction = "Up";
				break;
			case "ArrowLeft":
				direction = "Left";
				break;
			case "ArrowRight":
				direction = "Right";
				break;
			default:
				return;
		}

		if (direction) {
			gateway.send({
				Turn: {
					direction: direction,
				},
			});
		}
	};

	const renderTile = (tile, col_num, row_num) => {
		return (
			<div
				className="tile"
				key={arena_width * row_num + col_num}
				style={{
					backgroundColor: tile
						? `rgb(${tile["r"]},${tile["g"]},${tile["b"]})`
						: "rgb(37, 37, 37)",
				}}
			>
				({col_num}, {row_num})
			</div>
		);
	};

	const renderRow = (row, row_num) => {
		return (
			row && (
				<div className="arena-row" key={row_num}>
					{row.map((tile, col_num) => {
						return renderTile(tile, col_num, row_num);
					})}
					;
				</div>
			)
		);
	};

	const renderTiles = () => {
		let tiles = new Array(arena_height);
		for (let i = 0; i < arena_height; i++) {
			tiles[i] = new Array(arena_width);
			for (let j = 0; j < arena_width; j++) {
				tiles[i][j] = null;
			}
		}

		for (let point of food) {
			tiles[point.y][point.x] = { r: 0, g: 255, b: 0 };
		}

		for (let player of players) {
			for (let part of player["parts"]) {
				tiles[part.y][part.x] = player["colour"];
			}
		}

		return tiles.map(renderRow);
	};

	// console.debug(arena_height);

	return (
		<div id="arena" onKeyDown={(e) => handleKeyDown(e)} tabIndex="0">
			{arena_height && renderTiles()};
		</div>
	);
}
