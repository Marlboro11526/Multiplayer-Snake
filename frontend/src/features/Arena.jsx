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
	const uuid = useSelector((state) => state.userState.uuid);

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
			case "s":
				direction = "Down";
				break;
			case "ArrowUp":
			case "w":
				direction = "Up";
				break;
			case "ArrowLeft":
			case "a":
				direction = "Left";
				break;
			case "ArrowRight":
			case "d":
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
		const colour =
			tile && tile["colour"]
				? `rgb(${tile["colour"]["r"]},${tile["colour"]["g"]},${tile["colour"]["b"]})`
				: "rgb(0, 0, 0)";

		return (
			<div
				className="tile"
				key={arena_width * row_num + col_num}
				style={{
					backgroundColor: colour,
					boxShadow:
						tile && tile["uuid"] && uuid == tile["uuid"]
							? `0px 0px 5px 5px white`
							: "0px 0px 0px 0px",
				}}
			/>
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
			tiles[point.y][point.x] = { colour: { r: 0, g: 255, b: 0 } };
		}

		for (let player of players) {
			const player_snake = player[0];
			for (let part of player_snake["parts"]) {
				tiles[part.y][part.x] = {
					colour: player_snake["colour"],
					uuid: player[1],
				};
			}
		}

		return tiles.map(renderRow);
	};

	const renderLeaderboardEntry = (entry) => {
		const [colour, uuid, name, score] = entry;
		return (
			<div
				className="leaderboardEntry"
				key={uuid}
				style={{
					backgroundColor: `rgb(${colour["r"]},${colour["g"]},${colour["b"]})`,
				}}
			>
				{name} : {score}
			</div>
		);
	};

	const renderLeaderboard = () => {
		// 0, 1, 2, 3 = snake id, name, score =>
		// 0, 1, 2, 3 = colour, id, name, score (sorted)
		let playersResults = Array.from(
			players.map((entry) => [
				entry[0]["colour"],
				entry[1],
				entry[2],
				entry[3],
			])
		);
		playersResults.sort((lhs, rhs) => rhs[3] - lhs[3]);
		return playersResults.map(renderLeaderboardEntry);
	};

	return (
		<div id="arena_leaderboard_div">
			<aside id="leaderboard">
				<p id="leaderboard_header">Leaderboard</p>
				{players && renderLeaderboard(players)}
			</aside>
			<main id="arena" onKeyDown={(e) => handleKeyDown(e)} tabIndex="0">
				{arena_height && renderTiles()};
			</main>
		</div>
	);
}
